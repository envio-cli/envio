use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::time::Duration;

use super::{
    context::AppContext,
    navigation::NavigationStack,
    screens::{Action, Screen, ScreenEvent, ScreenId, SelectScreen},
};
use crate::{
    error::AppResult,
    utils::{get_profile_metadata, get_profile_path},
};

pub struct TuiApp {
    ctx: AppContext,
    navigation: NavigationStack,
    current_screen: Box<dyn Screen>,
    exit: bool,
}

impl TuiApp {
    pub fn default() -> AppResult<Self> {
        Ok(Self {
            ctx: AppContext::new(),
            navigation: NavigationStack::new(ScreenId::Select),
            current_screen: Box::new(SelectScreen::new()?),
            exit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        while !self.exit {
            terminal.draw(|f| {
                self.current_screen.draw(f);
                self.draw_banner(f);
            })?;

            if let Some(event) = self.current_screen.tick()? {
                self.handle_screen_event(event)?;
            }

            self.handle_input_events()?;
            self.update_current_screen()?;
            self.ctx.cache.cleanup_expired();

            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }

    fn draw_banner(&self, frame: &mut Frame) {
        let area = frame.area();
        let banner_text = "BETA";
        let banner_width = banner_text.len() as u16;
        let banner_area = Rect {
            x: area.width.saturating_sub(banner_width),
            y: area.height.saturating_sub(1),
            width: banner_width,
            height: 1,
        };

        let banner = Paragraph::new(Line::from(vec![Span::styled(
            banner_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Right);

        frame.render_widget(banner, banner_area);
    }

    fn update_current_screen(&mut self) -> AppResult<()> {
        if let Some(id) = self.navigation.current()
            && id != &self.current_screen.id()
        {
            self.current_screen = id.create_screen(&mut self.ctx)?;
        }
        Ok(())
    }

    fn handle_screen_event(&mut self, event: ScreenEvent) -> AppResult<()> {
        match event {
            ScreenEvent::ProfileDecrypted(profile) => {
                let name = profile.metadata.name.clone();
                self.ctx.cache.insert_profile(name.clone(), profile);
                self.navigation.push(ScreenId::Edit(name))?;
            }

            ScreenEvent::ProfileUpdated(profile) => {
                let name = profile.metadata.name.clone();
                self.ctx.cache.insert_profile(name, profile);
            }
        }
        Ok(())
    }

    fn handle_input_events(&mut self) -> AppResult<()> {
        if event::poll(Duration::from_millis(0))?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            self.handle_key(k)?;
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match self.current_screen.handle_key_event(key)? {
            Action::Exit => self.exit = true,
            Action::OpenProfile(name) => self.open_profile(&name)?,
            Action::NewProfile => self.navigation.push(ScreenId::CreateProfile)?,
            Action::EditProfile(name) => self.navigation.push(ScreenId::EditProfile(name))?,
            Action::Back => {
                let _ = self.navigation.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn open_profile(&mut self, name: &str) -> AppResult<()> {
        let metadata = get_profile_metadata(name)?;

        match metadata.cipher_kind {
            envio::cipher::CipherKind::PASSPHRASE | envio::cipher::CipherKind::AGE => {
                if self.ctx.cache.has_profile(name) {
                    self.navigation.push(ScreenId::Edit(name.to_string()))?;
                } else {
                    self.navigation
                        .push_overlay(ScreenId::GetKey(name.to_string()))?;
                }
            }
            _ => self.open_unencrypted_profile(name)?,
        }

        Ok(())
    }

    fn open_unencrypted_profile(&mut self, name: &str) -> AppResult<()> {
        let path = get_profile_path(name)?;
        let profile = envio::get_profile(path, None::<fn() -> String>)?;

        self.ctx.cache.insert_profile(name.to_string(), profile);
        self.navigation.push(ScreenId::Edit(name.to_string()))?;

        Ok(())
    }
}
