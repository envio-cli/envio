use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    DefaultTerminal, Frame,
};

use crate::error::AppResult;

use super::select::{Action as SelectAction, SelectScreen};

enum Screen {
    Select,
    Edit,
    Passphrase,
}

pub struct TuiApp {
    current_screen: Screen,
    select_screen: SelectScreen,
    exit: bool,
}

impl TuiApp {
    pub fn default() -> AppResult<Self> {
        Ok(TuiApp {
            current_screen: Screen::Select,
            select_screen: SelectScreen::new()?,
            exit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.current_screen {
            Screen::Select => {
                self.select_screen.draw(frame);
            }
            _ => {}
        }
    }

    fn handle_events(&mut self) -> AppResult<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AppResult<()> {
        match self.current_screen {
            Screen::Select => match self.select_screen.handle_key_event(key)? {
                SelectAction::Exit => {
                    self.exit = true;
                }

                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}
