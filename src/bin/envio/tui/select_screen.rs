use crate::{
    error::AppResult,
    utils::{get_configdir, get_profile_metadata},
};
use envio::{cipher::CipherKind, ProfileMetadata};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::screen::{Action, Screen};

fn styled_span(content: impl Into<String>, fg: Color, bold: bool) -> Span<'static> {
    let mut style = Style::default().fg(fg);

    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }

    Span::styled(content.into(), style)
}

#[derive(Clone)]
pub struct ProfileInfo {
    pub name: String,
    pub metadata: ProfileMetadata,
}

pub struct SelectScreen {
    profiles: Vec<ProfileInfo>,
    filtered_profiles: Vec<usize>,
    list_state: ListState,
    search_input: String,
    search_mode: bool,
}

impl Screen for SelectScreen {
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
        self.draw_profile_list(frame, chunks[1]);
        self.draw_footer(frame, chunks[2]);
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Action> {
        if self.search_mode {
            return self.handle_search_key(key);
        }

        match key.code {
            KeyCode::Char('/') => {
                self.search_mode = true;
                Ok(Action::None)
            }

            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
                Ok(Action::None)
            }

            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
                Ok(Action::None)
            }

            KeyCode::Char('n') => Ok(Action::NewProfile),

            KeyCode::Char('e') => Ok(self
                .get_selected_profile()
                .map_or(Action::None, |p| Action::EditProfile(p.name.clone()))),

            KeyCode::Char('d') => Ok(self
                .get_selected_profile()
                .map_or(Action::None, |p| Action::DeleteProfile(p.name.clone()))),

            KeyCode::Enter => Ok(self
                .get_selected_profile()
                .map_or(Action::None, |p| Action::OpenProfile(p.name.clone()))),

            KeyCode::Esc => Ok(Action::Exit),

            _ => Ok(Action::None),
        }
    }
}

impl SelectScreen {
    pub fn new() -> AppResult<Self> {
        let mut screen = Self {
            profiles: Vec::new(),
            filtered_profiles: Vec::new(),
            list_state: ListState::default(),
            search_input: String::new(),
            search_mode: false,
        };

        screen.load_profiles()?;
        screen.update_filter();

        if !screen.filtered_profiles.is_empty() {
            screen.list_state.select(Some(0));
        }

        Ok(screen)
    }

    pub fn load_profiles(&mut self) -> AppResult<()> {
        self.profiles.clear();
        let profile_dir = get_configdir().join("profiles");

        if !profile_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(profile_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("env") {
                continue;
            }

            let profile_name = match path.file_stem().and_then(|s| s.to_str()) {
                Some(name) if !name.starts_with('.') => name.to_string(),
                _ => continue,
            };

            let metadata = get_profile_metadata(&profile_name)?;

            self.profiles.push(ProfileInfo {
                name: profile_name,
                metadata,
            });
        }

        self.profiles
            .sort_by(|a, b| b.metadata.updated_at.cmp(&a.metadata.updated_at));

        Ok(())
    }

    fn update_filter(&mut self) {
        let search_lower = self.search_input.to_lowercase();

        if search_lower.is_empty() {
            self.filtered_profiles = (0..self.profiles.len()).collect();
        } else {
            self.filtered_profiles = self
                .profiles
                .iter()
                .enumerate()
                .filter(|(_, p)| p.name.to_lowercase().contains(&search_lower))
                .map(|(idx, _)| idx)
                .collect();
        }

        self.adjust_selection();
    }

    fn adjust_selection(&mut self) {
        let max_index = self.filtered_profiles.len().saturating_sub(1);
        let new_selection = self
            .list_state
            .selected()
            .filter(|&idx| idx <= max_index)
            .or_else(|| {
                if !self.filtered_profiles.is_empty() {
                    Some(0)
                } else {
                    None
                }
            });

        self.list_state.select(new_selection);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.search_mode {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            })
            .title(if self.search_mode {
                " Search (Esc to exit) "
            } else {
                " Search ('/' to search) "
            });

        let text = if self.search_input.is_empty() {
            vec![Line::from(Span::styled(
                "Type to search profiles...",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            vec![Line::from(self.search_input.as_str())]
        };

        frame.render_widget(Paragraph::new(text).block(block), area);
    }

    fn draw_profile_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .filtered_profiles
            .iter()
            .map(|&idx| {
                let p = &self.profiles[idx];
                let mut line = vec![styled_span(&p.name, Color::White, true)];

                if let Some(desc) = &p.metadata.description {
                    if !desc.is_empty() {
                        line.push(styled_span(&format!(" - {}", desc), Color::DarkGray, false));
                    }
                }

                let cipher_color = match p.metadata.cipher_kind {
                    CipherKind::PASSPHRASE => Color::Yellow,
                    CipherKind::GPG => Color::Green,
                    CipherKind::NONE => Color::Blue,
                };

                line.push(styled_span(
                    format!(" [{}]", p.metadata.cipher_kind),
                    cipher_color,
                    false,
                ));

                line.push(styled_span(
                    format!(
                        " (updated: {})",
                        p.metadata.updated_at.format("%Y-%m-%d %H:%M")
                    ),
                    Color::DarkGray,
                    false,
                ));

                ListItem::new(Line::from(line))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Profiles ({}) ", self.filtered_profiles.len()))
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let text = if self.search_mode {
            "Enter: Select | Esc: Exit search | Ctrl+C: Quit"
        } else {
            "Enter: Open | /: Search | n: New | e: Edit | d: Delete | Esc: Quit"
        };
        frame.render_widget(
            Paragraph::new(text)
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::TOP)),
            area,
        );
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> AppResult<Action> {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                Ok(Action::None)
            }

            KeyCode::Enter => {
                self.search_mode = false;
                Ok(self
                    .get_selected_profile()
                    .map_or(Action::None, |p| Action::OpenProfile(p.name.clone())))
            }

            KeyCode::Char(c) => {
                self.search_input.push(c);
                self.update_filter();
                Ok(Action::None)
            }

            KeyCode::Backspace => {
                self.search_input.pop();
                self.update_filter();
                Ok(Action::None)
            }

            _ => Ok(Action::None),
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_profiles.is_empty() {
            return;
        }

        let new = self.list_state.selected().unwrap_or(0) as i32 + delta;
        let new = new.clamp(0, self.filtered_profiles.len() as i32 - 1) as usize;

        self.list_state.select(Some(new));
    }

    pub fn get_selected_profile(&self) -> Option<&ProfileInfo> {
        self.list_state
            .selected()
            .and_then(|idx| self.filtered_profiles.get(idx))
            .map(|&i| &self.profiles[i])
    }
}
