use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    DefaultTerminal, Frame,
};

use crate::error::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Select,
    Edit,
}

pub struct TuiApp {
    screen: Screen,
    exit: bool,
}

impl TuiApp {
    pub fn default() -> Self {
        TuiApp {
            screen: Screen::Select,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.screen {
            Screen::Select => self.draw_select_screen(frame),
            Screen::Edit => self.draw_edit_screen(frame),
        }
    }

    fn draw_select_screen(&self, frame: &mut Frame) {
        frame.render_widget("select screen yoo", frame.area());
    }

    fn draw_edit_screen(&self, frame: &mut Frame) {
        frame.render_widget("edit screen", frame.area());
    }

    fn handle_events(&mut self) -> AppResult<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::Select => match key.code {
                KeyCode::Esc => {
                    self.exit = true;
                }
                _ => {}
            },

            Screen::Edit => match key.code {
                _ => {}
            },
        }
    }
}
