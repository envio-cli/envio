use envio::Profile;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::thread::{self, JoinHandle};
use zeroize::Zeroizing;

use super::{Action, Screen, ScreenEvent, ScreenId};
use crate::{error::AppResult, utils::get_profile_path};

enum Feedback {
    Decrypting,
    IncorrectKey,
    DecryptionFailed(String),
}

pub struct GetKeyScreen {
    profile_name: String,
    key: Zeroizing<String>,
    feedback: Option<Feedback>,
    decrypt_handle: Option<JoinHandle<Option<Profile>>>,
}

impl Screen for GetKeyScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Length(10),
                Constraint::Percentage(35),
            ])
            .split(area);

        let center_area = vertical_chunks[1];
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(center_area);

        let content_area = horizontal_chunks[1];
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(2),
            ])
            .split(content_area);

        let prompt = Paragraph::new(format!("Key for Profile: {}", self.profile_name))
            .style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(prompt, content_chunks[0]);

        let input_style = if self.key.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        };

        let input_display = if self.key.is_empty() {
            Span::styled("Type your secret key...", input_style)
        } else {
            Span::styled("*".repeat(self.key.len()), input_style)
        };

        let input = Paragraph::new(Line::from(input_display))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(Span::styled("Key Input", Style::default().fg(Color::Blue))),
            )
            .alignment(Alignment::Left);

        frame.render_widget(input, content_chunks[1]);
        frame.set_cursor_position((
            content_chunks[1].x + 1 + self.key.len() as u16,
            content_chunks[1].y + 1,
        ));

        if let Some(feedback) = &self.feedback {
            let (text, style) = match feedback {
                Feedback::Decrypting => ("Decrypting...", Style::default().fg(Color::Yellow)),
                Feedback::IncorrectKey => (
                    "Incorrect key. Please try again",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Feedback::DecryptionFailed(e) => (e.as_str(), Style::default().fg(Color::Red)),
            };

            let feedback_widget = Paragraph::new(text)
                .style(style)
                .alignment(Alignment::Center);
            frame.render_widget(feedback_widget, content_chunks[2]);
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Action> {
        match key.code {
            KeyCode::Enter => {
                self.feedback = Some(Feedback::Decrypting);
                self.spawn_decrypt()?;
            }

            KeyCode::Char(c) => {
                self.key.push(c);
                self.feedback = None;
            }

            KeyCode::Backspace => {
                self.key.pop();
                self.feedback = None;
            }

            KeyCode::Esc => return Ok(Action::Back),
            _ => {}
        }

        Ok(Action::None)
    }

    fn tick(&mut self) -> AppResult<Option<ScreenEvent>> {
        Ok(self.check_decrypt())
    }

    fn id(&self) -> ScreenId {
        ScreenId::GetKey(self.profile_name.clone())
    }
}

impl GetKeyScreen {
    pub fn new(profile_name: String) -> Self {
        Self {
            profile_name,
            key: Zeroizing::new(String::new()),
            feedback: None,
            decrypt_handle: None,
        }
    }

    fn spawn_decrypt(&mut self) -> AppResult<()> {
        let profile_name = self.profile_name.clone();
        let key = self.key.clone();

        self.decrypt_handle = Some(thread::spawn(move || {
            let profile_path = match get_profile_path(&profile_name) {
                Ok(p) => p,
                Err(_) => {
                    return None;
                }
            };

            let result = envio::get_profile(profile_path, Some(|| key)).ok();
            result
        }));

        Ok(())
    }

    fn check_decrypt(&mut self) -> Option<ScreenEvent> {
        if let Some(handle) = self.decrypt_handle.take() {
            match handle.join() {
                Ok(Some(profile)) => {
                    self.feedback = None;
                    return Some(ScreenEvent::ProfileDecrypted(profile));
                }

                Ok(None) => {
                    self.feedback = Some(Feedback::IncorrectKey);
                }

                Err(_) => {
                    self.feedback = Some(Feedback::DecryptionFailed(
                        "Decryption thread panicked".to_string(),
                    ));
                }
            }
        }

        None
    }
}
