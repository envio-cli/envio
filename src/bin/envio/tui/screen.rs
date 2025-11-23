use envio::Profile;
use ratatui::{crossterm::event::KeyEvent, Frame};

use crate::error::AppResult;

pub enum Action {
    None,
    Exit,
    OpenProfile(String),
    NewProfile,
    EditProfile(String),
    DeleteProfile(String),
    Back,
}

pub enum ScreenEvent {
    ProfileDecrypted(Profile),
}

pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Action>;
    fn tick(&mut self) -> AppResult<Option<ScreenEvent>> {
        Ok(None)
    }
}
