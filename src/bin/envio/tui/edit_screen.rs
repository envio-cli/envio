use envio::{Env, Profile};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::thread::{self, JoinHandle};

use super::screen::{Action, Screen};
use crate::{error::AppResult, tui::screen::ScreenEvent};

enum EditMode {
    None,
    Key(usize),
    Value(usize),
}

enum Status {
    Idle,
    Saving,
    Saved,
    Error(String, Color),
}

pub struct EditScreen {
    profile: Profile,
    envs: Vec<Env>,
    list_state: ListState,
    edit_mode: EditMode,
    edit_buffer: String,
    status: Status,
    save_handle: Option<JoinHandle<AppResult<()>>>,
}

impl EditScreen {
    pub fn new(profile: Profile) -> Self {
        let envs: Vec<Env> = profile.envs.iter().cloned().collect();
        let mut list_state = ListState::default();
        if !envs.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            profile,
            envs,
            list_state,
            edit_mode: EditMode::None,
            edit_buffer: String::new(),
            status: Status::Idle,
            save_handle: None,
        }
    }

    fn get_selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn move_selection(&mut self, delta: i32) {
        if self.envs.is_empty() {
            return;
        }

        let current = self.get_selected_index().unwrap_or(0) as i32;
        let new = (current + delta).clamp(0, self.envs.len() as i32 - 1) as usize;
        self.list_state.select(Some(new));
    }

    fn start_edit_key(&mut self) {
        if let Some(idx) = self.get_selected_index() {
            if idx < self.envs.len() {
                self.edit_buffer = self.envs[idx].name.clone();
                self.edit_mode = EditMode::Key(idx);
            }
        }
    }

    fn start_edit_value(&mut self) {
        if let Some(idx) = self.get_selected_index() {
            if idx < self.envs.len() {
                self.edit_buffer = self.envs[idx].value.clone();
                self.edit_mode = EditMode::Value(idx);
            }
        }
    }

    fn finish_edit(&mut self) {
        match self.edit_mode {
            EditMode::Key(idx) if idx < self.envs.len() => {
                let trimmed = self.edit_buffer.trim().to_string();

                if trimmed.is_empty() {
                    if self.envs[idx].name.is_empty() && self.envs[idx].value.is_empty() {
                        self.envs.remove(idx);

                        if !self.envs.is_empty() {
                            let new_idx = idx.min(self.envs.len().saturating_sub(1));
                            self.list_state.select(Some(new_idx));
                        } else {
                            self.list_state.select(None);
                        }
                    }
                } else {
                    self.envs[idx].name = trimmed;
                }
            }

            EditMode::Value(idx) if idx < self.envs.len() => {
                self.envs[idx].value = self.edit_buffer.clone();
            }
            _ => {}
        }

        self.edit_mode = EditMode::None;
        self.edit_buffer.clear();
    }

    fn cancel_edit(&mut self) {
        if let EditMode::Key(idx) = self.edit_mode {
            if idx < self.envs.len()
                && self.envs[idx].name.is_empty()
                && self.envs[idx].value.is_empty()
            {
                self.envs.remove(idx);
                if !self.envs.is_empty() {
                    let new_idx = idx.min(self.envs.len().saturating_sub(1));
                    self.list_state.select(Some(new_idx));
                } else {
                    self.list_state.select(None);
                }
            }
        }

        self.edit_mode = EditMode::None;
        self.edit_buffer.clear();
    }

    fn add_new_pair(&mut self) {
        let new_env = Env::from_key_value("", "");
        self.envs.push(new_env);

        let new_idx = self.envs.len() - 1;
        self.list_state.select(Some(new_idx));

        self.start_edit_key();
    }

    fn delete_current(&mut self) {
        if let Some(idx) = self.get_selected_index() {
            if idx < self.envs.len() {
                self.envs.remove(idx);

                if self.envs.is_empty() {
                    self.list_state.select(None);
                } else {
                    let new_idx = (idx).min(self.envs.len().saturating_sub(1));
                    self.list_state.select(Some(new_idx));
                }
            }
        }
    }

    fn save_changes(&mut self) -> AppResult<()> {
        self.profile.envs = self.envs.clone().into();

        let mut profile = self.profile.clone();

        self.status = Status::Saving;
        self.save_handle = Some(thread::spawn(move || {
            profile.save()?;
            Ok(())
        }));

        Ok(())
    }

    fn check_save(&mut self) {
        if let Some(handle) = self.save_handle.take() {
            match handle.join() {
                Ok(Ok(())) => self.status = Status::Saved,
                Ok(Err(e)) => self.status = Status::Error(e.to_string(), Color::Red),
                Err(_) => {
                    self.status = Status::Error("Save thread panicked".to_string(), Color::Red)
                }
            }
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let name = &self.profile.metadata.name;
        let description = self.profile.metadata.description.as_deref().unwrap_or("");

        let title = if description.is_empty() {
            format!(" {} ", name)
        } else {
            format!(" {} - {} ", name, description)
        };

        let border_color = match &self.status {
            Status::Saving => Color::Yellow,
            Status::Saved => Color::Green,
            Status::Error(_, color) => *color,
            Status::Idle => Color::Blue,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));

        let status_text = match &self.status {
            Status::Saving => Span::styled("Saving...", Style::default().fg(Color::Yellow)),
            Status::Saved => Span::styled("Saved", Style::default().fg(Color::Green)),
            Status::Error(e, color) => {
                Span::styled(format!("Save error: {}", e), Style::default().fg(*color))
            }
            Status::Idle => Span::styled(
                format!("{} environment variables", self.envs.len()),
                Style::default().fg(Color::DarkGray),
            ),
        };

        let text = vec![Line::from(status_text)];

        frame.render_widget(Paragraph::new(text).block(block), area);
    }

    fn draw_env_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .envs
            .iter()
            .enumerate()
            .map(|(idx, env)| {
                let is_editing_key = matches!(self.edit_mode, EditMode::Key(i) if i == idx);
                let is_editing_value = matches!(self.edit_mode, EditMode::Value(i) if i == idx);
                let is_selected = self.get_selected_index() == Some(idx);

                let key_display = if is_editing_key {
                    self.edit_buffer.as_str()
                } else {
                    &env.name
                };

                let value_display = if is_editing_value {
                    self.edit_buffer.as_str()
                } else {
                    &env.value
                };

                let key_style = if is_editing_key {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let value_style = if is_editing_value {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else if is_selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let line = vec![
                    Span::styled(format!("{}", key_display), key_style),
                    Span::styled(" = ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{}", value_display), value_style),
                ];

                ListItem::new(Line::from(line))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Environment Variables ")
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);

        if let Some(idx) = self.get_selected_index() {
            if idx >= self.envs.len() {
                return;
            }

            let item_y = area.y + 1 + idx as u16;
            let base_x = area.x + 3; // +1 for border, +2 for symbol

            let cursor_x = match &self.edit_mode {
                EditMode::Key(_) => base_x + self.edit_buffer.len() as u16,
                EditMode::Value(_) => {
                    let key_len = self.envs[idx].name.len() as u16;

                    // base_x + key_len + 3 spacing between key and value
                    base_x + key_len + 3 + self.edit_buffer.len() as u16
                }
                _ => return,
            };

            let bounded_x = cursor_x.min(area.x + area.width - 2);
            frame.set_cursor_position((bounded_x, item_y));
        }
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let text = match self.edit_mode {
            EditMode::Key(_) => "Editing key: Type to edit | Enter: Finish | Esc: Cancel | Left/Right: Switch",
            EditMode::Value(_) => "Editing value: Type to edit | Enter: Finish | Esc: Cancel | Left/Right: Switch",
            _ => "↑↓: Navigate | Enter: Edit key | →: Edit value | a: Add | d: Delete | s: Save | Esc: Back",
        };

        frame.render_widget(
            Paragraph::new(text)
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::TOP)),
            area,
        );
    }
}

impl Screen for EditScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
            .split(area);

        self.draw_header(frame, chunks[0]);
        self.draw_env_list(frame, chunks[1]);
        self.draw_footer(frame, chunks[2]);
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Action> {
        if !matches!(self.status, Status::Saving) {
            self.status = Status::Idle;
        }

        match self.edit_mode {
            EditMode::Key(_) | EditMode::Value(_) => match key.code {
                KeyCode::Enter => {
                    self.finish_edit();
                    Ok(Action::None)
                }

                KeyCode::Esc => {
                    self.cancel_edit();
                    Ok(Action::None)
                }

                KeyCode::Left => {
                    if matches!(self.edit_mode, EditMode::Value(_)) {
                        self.finish_edit();
                        self.start_edit_key();
                    }

                    Ok(Action::None)
                }

                KeyCode::Right => {
                    if matches!(self.edit_mode, EditMode::Key(_)) {
                        self.finish_edit();
                        self.start_edit_value();
                    }

                    Ok(Action::None)
                }

                KeyCode::Char(c) => {
                    self.edit_buffer.push(c);
                    Ok(Action::None)
                }

                KeyCode::Backspace => {
                    self.edit_buffer.pop();
                    Ok(Action::None)
                }

                _ => Ok(Action::None),
            },

            EditMode::None => match key.code {
                KeyCode::Esc => Ok(Action::Back),

                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_selection(-1);
                    Ok(Action::None)
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_selection(1);
                    Ok(Action::None)
                }

                KeyCode::Enter => {
                    self.start_edit_key();
                    Ok(Action::None)
                }

                KeyCode::Right => {
                    self.start_edit_value();
                    Ok(Action::None)
                }

                KeyCode::Char('a') => {
                    self.add_new_pair();
                    Ok(Action::None)
                }

                KeyCode::Char('d') => {
                    self.delete_current();
                    Ok(Action::None)
                }

                KeyCode::Char('s') => {
                    self.save_changes()?;
                    Ok(Action::None)
                }

                _ => Ok(Action::None),
            },
        }
    }

    fn tick(&mut self) -> AppResult<Option<ScreenEvent>> {
        self.check_save();
        Ok(None)
    }
}
