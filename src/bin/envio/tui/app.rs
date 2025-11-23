use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    DefaultTerminal, Frame,
};
use std::time::Duration;

use crate::{
    error::AppResult,
    tui::{
        edit_screen::EditScreen,
        get_key_screen::GetKeyScreen,
        profile_form_screen::{CreateProfileScreen, EditProfileScreen},
        screen::{Action, Screen, ScreenEvent},
        select_screen::SelectScreen,
    },
    utils::{get_profile_metadata, get_profile_path},
};

pub struct TuiApp {
    current_screen: Box<dyn Screen>,
    exit: bool,
}

impl TuiApp {
    pub fn default() -> AppResult<Self> {
        Ok(TuiApp {
            current_screen: Box::new(SelectScreen::new()?),
            exit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if let Some(event) = self.current_screen.tick()? {
                self.handle_screen_event(event)?;
            }

            self.handle_events()?;

            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.current_screen.draw(frame);
    }

    fn handle_screen_event(&mut self, event: ScreenEvent) -> AppResult<()> {
        match event {
            ScreenEvent::ProfileDecrypted(profile) => {
                self.current_screen = Box::new(EditScreen::new(profile));
            }
        }

        Ok(())
    }

    fn handle_events(&mut self) -> AppResult<()> {
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<()> {
        let action = self.current_screen.handle_key_event(key)?;

        match action {
            Action::Exit => {
                self.exit = true;
            }

            Action::OpenProfile(profile_name) => {
                let metadata = get_profile_metadata(&profile_name)?;

                if metadata.cipher_kind == envio::cipher::CipherKind::PASSPHRASE {
                    self.current_screen = Box::new(GetKeyScreen::new(profile_name));
                } else {
                    let profile_path = get_profile_path(&profile_name)?;
                    let profile = envio::get_profile(profile_path, None::<fn() -> String>)?;
                    self.current_screen = Box::new(EditScreen::new(profile));
                }
            }

            Action::NewProfile => {
                self.current_screen = Box::new(CreateProfileScreen::new()?);
            }

            Action::EditProfile(profile_name) => {
                self.current_screen = Box::new(EditProfileScreen::new(profile_name)?);
            }

            Action::Back => {
                self.current_screen = Box::new(SelectScreen::new()?);
            }

            _ => {}
        }

        Ok(())
    }
}
