mod edit_screen;
mod get_key_screen;
mod profile_form_screen;
mod select_screen;

pub use edit_screen::EditScreen;
pub use get_key_screen::GetKeyScreen;
pub use profile_form_screen::{CreateProfileScreen, EditProfileScreen};
pub use select_screen::SelectScreen;

use envio::Profile;
use ratatui::{crossterm::event::KeyEvent, Frame};

use crate::{error::AppResult, tui::context::AppContext};

pub enum Action {
    None,
    Exit,
    OpenProfile(String),
    NewProfile,
    EditProfile(String),
    Back,
}

pub enum ScreenEvent {
    ProfileDecrypted(Profile),
    ProfileUpdated(Profile),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScreenId {
    Select,
    CreateProfile,
    EditProfile(String),
    GetKey(String),
    Edit(String),
}

impl ScreenId {
    pub fn create_screen(&self, context: &mut AppContext) -> AppResult<Box<dyn Screen>> {
        match self {
            ScreenId::Select => Ok(Box::new(SelectScreen::new()?)),
            ScreenId::CreateProfile => Ok(Box::new(CreateProfileScreen::new()?)),
            ScreenId::EditProfile(name) => Ok(Box::new(EditProfileScreen::new(name.clone())?)),
            ScreenId::GetKey(name) => Ok(Box::new(GetKeyScreen::new(name.clone()))),
            ScreenId::Edit(name) => Ok(Box::new(EditScreen::new(name.clone(), context)?)),
        }
    }
}

pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<Action>;
    fn tick(&mut self) -> AppResult<Option<ScreenEvent>> {
        Ok(None)
    }

    fn id(&self) -> ScreenId;
}
