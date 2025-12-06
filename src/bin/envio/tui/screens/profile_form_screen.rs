use envio::{
    EnvMap,
    cipher::{CipherKind, create_cipher, gpg::get_gpg_keys},
};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::thread::{self, JoinHandle};
use strum::IntoEnumIterator;
use zeroize::Zeroizing;

use super::{Action, Screen, ScreenEvent, ScreenId};
use crate::{
    error::AppResult,
    ops,
    utils::{build_profile_path, get_profile_metadata, get_profile_path},
};

enum Status {
    Idle,
    Saving,
    Saved,
    Error(String, Color),
}

fn draw_text_field(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    placeholder: &str,
    text: &str,
    is_active: bool,
) {
    let border_color = if is_active { Color::Cyan } else { Color::Gray };
    let title_style = if is_active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let display_text = if text.is_empty() {
        Span::styled(placeholder, Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(text, Style::default().fg(Color::White))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, title_style))
        .border_style(Style::default().fg(border_color));

    frame.render_widget(Paragraph::new(Line::from(display_text)).block(block), area);

    if is_active {
        let cursor_x = area.x + 1 + text.len() as u16;
        let cursor_x = cursor_x.min(area.x + area.width - 2);
        frame.set_cursor_position((cursor_x, area.y + 1));
    }
}

fn draw_header(frame: &mut Frame, area: Rect, title: &str, status: &Status, status_message: &str) {
    let border_color = match status {
        Status::Saving => Color::Yellow,
        Status::Saved => Color::Green,
        Status::Error(_, color) => *color,
        Status::Idle => Color::Blue,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border_color));

    let status_text = match status {
        Status::Saving => Span::styled("Saving...", Style::default().fg(Color::Yellow)),
        Status::Saved => Span::styled("Saved", Style::default().fg(Color::Green)),
        Status::Error(e, color) => {
            Span::styled(format!("Error: {}", e), Style::default().fg(*color))
        }
        Status::Idle => Span::styled(status_message, Style::default().fg(Color::DarkGray)),
    };

    let text = vec![Line::from(status_text)];

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_footer(frame: &mut Frame, area: Rect, text: &str) {
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP)),
        area,
    );
}

enum CreateField {
    Name,
    Description,
    CipherKind,
    Key,
}

enum KeySubField {
    Passphrase,
    PassphraseConfirm,
    Gpg,
}

pub struct CreateProfileScreen {
    name: String,
    description: String,
    cipher_kind_list_state: ListState,
    cipher_kinds: Vec<CipherKind>,
    passphrase: Zeroizing<String>,
    passphrase_confirm: Zeroizing<String>,
    gpg_keys: Vec<(String, String)>,
    gpg_key_list_state: ListState,
    current_field: CreateField,
    key_sub_field: KeySubField,
    status: Status,
    save_handle: Option<JoinHandle<AppResult<()>>>,
}

impl Screen for CreateProfileScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        draw_header(
            frame,
            chunks[0],
            " Create New Profile ",
            &self.status,
            "Fill in the form to create a new profile",
        );

        let needs_key = self.needs_key_input();
        let key_height = if needs_key {
            match self.get_selected_cipher_kind() {
                CipherKind::PASSPHRASE => 6,
                CipherKind::GPG => 8,
                _ => 0,
            }
        } else {
            0
        };

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length(key_height),
                Constraint::Min(0),
            ])
            .split(chunks[1]);

        draw_text_field(
            frame,
            form_chunks[0],
            " Name ",
            "Enter profile name...",
            &self.name,
            matches!(self.current_field, CreateField::Name),
        );

        draw_text_field(
            frame,
            form_chunks[1],
            " Description ",
            "Enter description (optional)...",
            &self.description,
            matches!(self.current_field, CreateField::Description),
        );

        self.draw_cipher_kind_field(frame, form_chunks[2]);
        if needs_key {
            self.draw_key_field(frame, form_chunks[3]);
        }

        let footer_text = match self.current_field {
            CreateField::CipherKind => "↑↓: Select | Tab/Enter: Next field | Esc: Cancel",
            CreateField::Key if matches!(self.get_selected_cipher_kind(), CipherKind::GPG) => {
                "↑↓: Select | Tab/Enter: Next field | Shift+Tab: Previous field | Esc: Cancel"
            }
            CreateField::Key => "Tab/Enter: Next field | Shift+Tab: Previous field | Esc: Cancel",
            _ => "Tab/Enter: Next field | Shift+Tab: Previous field | Ctrl+s: Save | Esc: Cancel",
        };

        draw_footer(frame, chunks[2], footer_text);
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Action> {
        if !matches!(self.status, Status::Saving) {
            self.status = Status::Idle;
        }

        match self.current_field {
            CreateField::CipherKind => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_cipher_selection(-1);
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_cipher_selection(1);
                }

                KeyCode::Tab | KeyCode::Enter => {
                    self.move_to_next_field();
                }

                KeyCode::BackTab => {
                    self.move_to_previous_field();
                }

                KeyCode::Esc => return Ok(Action::Back),

                _ => {}
            },

            CreateField::Key => match self.get_selected_cipher_kind() {
                CipherKind::GPG => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.move_gpg_selection(-1);
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        self.move_gpg_selection(1);
                    }

                    KeyCode::Tab | KeyCode::Enter => {
                        self.move_to_next_field();
                    }

                    KeyCode::BackTab => {
                        self.move_to_previous_field();
                    }

                    KeyCode::Esc => return Ok(Action::Back),

                    _ => {}
                },

                CipherKind::PASSPHRASE => match key.code {
                    KeyCode::Tab | KeyCode::Enter => match self.key_sub_field {
                        KeySubField::Passphrase => {
                            if !self.passphrase.is_empty() {
                                self.key_sub_field = KeySubField::PassphraseConfirm;
                            }
                        }

                        KeySubField::PassphraseConfirm => {
                            self.move_to_next_field();
                        }
                        _ => {}
                    },

                    KeyCode::BackTab => match self.key_sub_field {
                        KeySubField::PassphraseConfirm => {
                            self.key_sub_field = KeySubField::Passphrase;
                        }
                        KeySubField::Passphrase => {
                            self.move_to_previous_field();
                        }
                        _ => {}
                    },

                    KeyCode::Char('s')
                        if key
                            .modifiers
                            .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        self.save()?;
                    }

                    KeyCode::Char(c) => match self.key_sub_field {
                        KeySubField::Passphrase => {
                            self.passphrase.push(c);
                        }
                        KeySubField::PassphraseConfirm => {
                            self.passphrase_confirm.push(c);
                        }
                        _ => {}
                    },

                    KeyCode::Backspace => match self.key_sub_field {
                        KeySubField::Passphrase => {
                            self.passphrase.pop();
                        }
                        KeySubField::PassphraseConfirm => {
                            self.passphrase_confirm.pop();
                        }
                        _ => {}
                    },

                    KeyCode::Esc => return Ok(Action::Back),

                    _ => {}
                },

                _ => {}
            },

            CreateField::Name | CreateField::Description => match key.code {
                KeyCode::Tab | KeyCode::Enter => {
                    self.move_to_next_field();
                }

                KeyCode::BackTab => {
                    self.move_to_previous_field();
                }

                KeyCode::Char('s')
                    if key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
                {
                    self.save()?;
                }

                KeyCode::Char(c) => match self.current_field {
                    CreateField::Name => {
                        if c != ' ' {
                            self.name.push(c);
                        }
                    }

                    CreateField::Description => self.description.push(c),
                    _ => {}
                },

                KeyCode::Backspace => match self.current_field {
                    CreateField::Name => {
                        self.name.pop();
                    }

                    CreateField::Description => {
                        self.description.pop();
                    }
                    _ => {}
                },

                KeyCode::Esc => return Ok(Action::Back),

                _ => {}
            },
        }

        Ok(Action::None)
    }

    fn tick(&mut self) -> AppResult<Option<ScreenEvent>> {
        self.check_save();
        Ok(None)
    }

    fn id(&self) -> ScreenId {
        ScreenId::CreateProfile
    }
}

impl CreateProfileScreen {
    pub fn new() -> AppResult<Self> {
        let cipher_kinds: Vec<CipherKind> = CipherKind::iter().collect();
        let mut list_state = ListState::default();

        if !cipher_kinds.is_empty() {
            list_state.select(Some(0));
        }

        let gpg_keys = get_gpg_keys()?;
        let mut gpg_key_list_state = ListState::default();
        if !gpg_keys.is_empty() {
            gpg_key_list_state.select(Some(0));
        }

        Ok(Self {
            name: String::new(),
            description: String::new(),
            cipher_kind_list_state: list_state,
            cipher_kinds,
            passphrase: Zeroizing::new(String::new()),
            passphrase_confirm: Zeroizing::new(String::new()),
            gpg_keys,
            gpg_key_list_state,
            current_field: CreateField::Name,
            key_sub_field: KeySubField::Passphrase,
            status: Status::Idle,
            save_handle: None,
        })
    }

    fn needs_key_input(&self) -> bool {
        matches!(
            self.get_selected_cipher_kind(),
            CipherKind::PASSPHRASE | CipherKind::GPG
        )
    }

    fn move_to_next_field(&mut self) {
        self.current_field = match self.current_field {
            CreateField::Name => CreateField::Description,
            CreateField::Description => CreateField::CipherKind,
            CreateField::CipherKind => {
                if self.needs_key_input() {
                    self.key_sub_field = match self.get_selected_cipher_kind() {
                        CipherKind::GPG => KeySubField::Gpg,
                        _ => KeySubField::Passphrase,
                    };
                    CreateField::Key
                } else {
                    CreateField::Name
                }
            }
            CreateField::Key => CreateField::Name,
        };
    }

    fn move_to_previous_field(&mut self) {
        self.current_field = match self.current_field {
            CreateField::Name => {
                if self.needs_key_input() {
                    CreateField::Key
                } else {
                    CreateField::CipherKind
                }
            }
            CreateField::Description => CreateField::Name,
            CreateField::CipherKind => CreateField::Description,
            CreateField::Key => CreateField::CipherKind,
        };
    }

    fn get_selected_cipher_kind(&self) -> CipherKind {
        self.cipher_kinds[self.cipher_kind_list_state.selected().unwrap_or(0)]
    }

    fn get_selected_gpg_key(&self) -> Option<String> {
        if self.gpg_keys.is_empty() {
            return None;
        }

        Some(
            self.gpg_keys[self.gpg_key_list_state.selected().unwrap_or(0)]
                .1
                .clone(),
        )
    }

    fn move_cipher_selection(&mut self, delta: i32) {
        if self.cipher_kinds.is_empty() {
            return;
        }

        let current = self.cipher_kind_list_state.selected().unwrap_or(0) as i32;
        let new = (current + delta).clamp(0, self.cipher_kinds.len() as i32 - 1) as usize;
        self.cipher_kind_list_state.select(Some(new));

        if matches!(self.get_selected_cipher_kind(), CipherKind::GPG) {
            self.key_sub_field = KeySubField::Gpg;
        } else {
            self.passphrase.clear();
            self.passphrase_confirm.clear();
            self.key_sub_field = KeySubField::Passphrase;
        }
    }

    fn move_gpg_selection(&mut self, delta: i32) {
        if self.gpg_keys.is_empty() {
            return;
        }

        let current = self.gpg_key_list_state.selected().unwrap_or(0) as i32;
        let new = (current + delta).clamp(0, self.gpg_keys.len() as i32 - 1) as usize;
        self.gpg_key_list_state.select(Some(new));
    }

    fn save(&mut self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            self.status = Status::Error("Name cannot be empty".to_string(), Color::Red);
            return Ok(());
        }

        if self.name.trim().contains(' ') {
            self.status =
                Status::Error("Profile name cannot contain spaces".to_string(), Color::Red);
            return Ok(());
        }

        let key = match self.get_selected_cipher_kind() {
            CipherKind::PASSPHRASE | CipherKind::AGE => {
                if self.passphrase.len() < 8 {
                    self.status = Status::Error(
                        "Passphrase must be at least 8 characters".to_string(),
                        Color::Red,
                    );
                    return Ok(());
                }

                if self.passphrase != self.passphrase_confirm {
                    self.status = Status::Error("Passphrases do not match".to_string(), Color::Red);
                    return Ok(());
                }

                Some(self.passphrase.clone())
            }

            CipherKind::GPG => {
                if let Some(ref key) = self.get_selected_gpg_key() {
                    Some(key.clone().into())
                } else {
                    self.status = Status::Error("Please select a GPG key".to_string(), Color::Red);
                    return Ok(());
                }
            }

            CipherKind::NONE => None,
        };

        self.status = Status::Saving;

        let name = self.name.trim().to_string();
        let description = self.description.trim().to_string();
        let cipher_kind = self.get_selected_cipher_kind();
        self.save_handle = Some(thread::spawn(move || {
            ops::create_profile(
                name,
                Some(description),
                EnvMap::default(),
                create_cipher(cipher_kind, key)?,
            )?;

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

    fn draw_cipher_kind_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_active = matches!(self.current_field, CreateField::CipherKind);
        let border_color = if is_active { Color::Cyan } else { Color::Gray };
        let title_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let items: Vec<ListItem> = self
            .cipher_kinds
            .iter()
            .enumerate()
            .map(|(idx, kind)| {
                let is_selected = self.cipher_kind_list_state.selected() == Some(idx);
                let style = if is_selected && is_active {
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

                let cipher_color = match kind {
                    CipherKind::PASSPHRASE => Color::Yellow,
                    CipherKind::AGE => Color::Magenta,
                    CipherKind::GPG => Color::Green,
                    CipherKind::NONE => Color::Blue,
                };

                let line = vec![Span::styled(kind.to_string(), style.fg(cipher_color))];

                ListItem::new(Line::from(line))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(" Encryption Type ", title_style))
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.cipher_kind_list_state);
    }

    fn draw_key_field(&mut self, frame: &mut Frame, area: Rect) {
        let is_active = matches!(self.current_field, CreateField::Key);
        let border_color = if is_active { Color::Cyan } else { Color::Gray };
        let title_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        match self.get_selected_cipher_kind() {
            CipherKind::PASSPHRASE => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)])
                    .split(area);

                let passphrase_display = if self.passphrase.is_empty() {
                    Span::styled(
                        "Enter passphrase (min 8 chars)...",
                        Style::default().fg(Color::DarkGray),
                    )
                } else {
                    Span::styled(
                        "*".repeat(self.passphrase.len()),
                        Style::default().fg(Color::White),
                    )
                };

                let is_passphrase_active = matches!(self.current_field, CreateField::Key)
                    && matches!(self.key_sub_field, KeySubField::Passphrase);
                let is_confirm_active = matches!(self.current_field, CreateField::Key)
                    && matches!(self.key_sub_field, KeySubField::PassphraseConfirm);

                let passphrase_block = Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        " Passphrase ",
                        if is_passphrase_active {
                            title_style
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ))
                    .border_style(Style::default().fg(if is_passphrase_active {
                        border_color
                    } else {
                        Color::Gray
                    }));

                frame.render_widget(
                    Paragraph::new(Line::from(passphrase_display)).block(passphrase_block),
                    chunks[0],
                );

                if is_passphrase_active {
                    let cursor_x = chunks[0].x + 1 + self.passphrase.len() as u16;
                    let cursor_x = cursor_x.min(chunks[0].x + chunks[0].width - 2);
                    frame.set_cursor_position((cursor_x, chunks[0].y + 1));
                }

                let confirm_display = if self.passphrase_confirm.is_empty() {
                    Span::styled(
                        "Confirm passphrase...",
                        Style::default().fg(Color::DarkGray),
                    )
                } else {
                    Span::styled(
                        "*".repeat(self.passphrase_confirm.len()),
                        Style::default().fg(Color::White),
                    )
                };

                let confirm_block = Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        " Confirm Passphrase ",
                        if is_confirm_active {
                            title_style
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ))
                    .border_style(Style::default().fg(if is_confirm_active {
                        border_color
                    } else {
                        Color::Gray
                    }));

                frame.render_widget(
                    Paragraph::new(Line::from(confirm_display)).block(confirm_block),
                    chunks[1],
                );

                if is_confirm_active {
                    let cursor_x = chunks[1].x + 1 + self.passphrase_confirm.len() as u16;
                    let cursor_x = cursor_x.min(chunks[1].x + chunks[1].width - 2);
                    frame.set_cursor_position((cursor_x, chunks[1].y + 1));
                }
            }

            CipherKind::GPG => {
                if self.gpg_keys.is_empty() {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled(" GPG Key ", title_style))
                        .border_style(Style::default().fg(border_color));

                    let text = Span::styled(
                        "No GPG keys found. Please install GPG and generate a key.",
                        Style::default().fg(Color::Red),
                    );

                    frame.render_widget(Paragraph::new(Line::from(text)).block(block), area);
                    return;
                }

                let items: Vec<ListItem> = self
                    .gpg_keys
                    .iter()
                    .enumerate()
                    .map(|(idx, (name, _))| {
                        let is_selected = self.gpg_key_list_state.selected() == Some(idx);
                        let style = if is_selected && is_active {
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

                        ListItem::new(Line::from(Span::styled(name.clone(), style)))
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(Span::styled(" GPG Key ", title_style))
                            .border_style(Style::default().fg(border_color)),
                    )
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                    .highlight_symbol("▶ ");

                frame.render_stateful_widget(list, area, &mut self.gpg_key_list_state);
            }

            _ => {}
        }
    }
}

enum EditField {
    Name,
    Description,
}

pub struct EditProfileScreen {
    profile_name: String,
    name: String,
    description: String,
    current_field: EditField,
    status: Status,
}

impl Screen for EditProfileScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        draw_header(
            frame,
            chunks[0],
            " Edit Profile ",
            &self.status,
            "Edit profile metadata",
        );

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(chunks[1]);

        draw_text_field(
            frame,
            form_chunks[0],
            " Name ",
            "Enter profile name...",
            &self.name,
            matches!(self.current_field, EditField::Name),
        );

        draw_text_field(
            frame,
            form_chunks[1],
            " Description ",
            "Enter description (optional)...",
            &self.description,
            matches!(self.current_field, EditField::Description),
        );

        draw_footer(
            frame,
            chunks[2],
            "Tab/Enter: Next field | Shift+Tab: Previous field | Ctrl+s: Save | Esc: Cancel",
        );
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Action> {
        if !matches!(self.status, Status::Saving) {
            self.status = Status::Idle;
        }

        match key.code {
            KeyCode::Tab | KeyCode::Enter => {
                self.move_to_next_field();
            }

            KeyCode::BackTab => {
                self.move_to_previous_field();
            }

            KeyCode::Char('s')
                if key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.save()?;
            }

            KeyCode::Char(c) => match self.current_field {
                EditField::Name => {
                    if c != ' ' {
                        self.name.push(c);
                    }
                }
                EditField::Description => self.description.push(c),
            },

            KeyCode::Backspace => match self.current_field {
                EditField::Name => {
                    self.name.pop();
                }
                EditField::Description => {
                    self.description.pop();
                }
            },

            KeyCode::Esc => return Ok(Action::Back),

            _ => {}
        }

        Ok(Action::None)
    }

    fn id(&self) -> ScreenId {
        ScreenId::EditProfile(self.profile_name.clone())
    }
}

impl EditProfileScreen {
    pub fn new(profile_name: String) -> AppResult<Self> {
        let metadata = get_profile_metadata(&profile_name)?;
        Ok(Self {
            profile_name,
            name: metadata.name.clone(),
            description: metadata.description.unwrap_or_default(),
            current_field: EditField::Name,
            status: Status::Idle,
        })
    }

    fn move_to_next_field(&mut self) {
        self.current_field = match self.current_field {
            EditField::Name => EditField::Description,
            EditField::Description => EditField::Name,
        };
    }

    fn move_to_previous_field(&mut self) {
        self.current_field = match self.current_field {
            EditField::Name => EditField::Description,
            EditField::Description => EditField::Name,
        };
    }

    fn save(&mut self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            self.status = Status::Error("Name cannot be empty".to_string(), Color::Red);
            return Ok(());
        }

        if self.name.trim().contains(' ') {
            self.status =
                Status::Error("Profile name cannot contain spaces".to_string(), Color::Red);
            return Ok(());
        }

        let new_profile_name = self.name.trim().to_string();
        let new_profile_description = if self.description.trim().is_empty() {
            None
        } else {
            Some(self.description.trim().to_string())
        };

        let new_file_path = build_profile_path(&new_profile_name);
        let old_file_path = get_profile_path(&self.profile_name)?;

        let mut serialized_profile = envio::utils::get_serialized_profile(&old_file_path)?;

        serialized_profile.metadata.name = new_profile_name;
        serialized_profile.metadata.description = new_profile_description;
        serialized_profile.metadata.file_path = new_file_path.clone();

        envio::utils::save_serialized_profile(&old_file_path, serialized_profile)?;

        std::fs::rename(old_file_path, new_file_path)?;

        self.status = Status::Saved;

        Ok(())
    }
}
