use crate::{error::AppResult, tui::screens::ScreenId};

pub struct NavigationStack {
    stack: Vec<ScreenId>,
    overlay: Option<ScreenId>,
    max_depth: usize,
}

impl NavigationStack {
    pub fn new(initial_screen: ScreenId) -> Self {
        Self {
            stack: vec![initial_screen],
            overlay: None,
            max_depth: 50,
        }
    }

    pub fn current(&self) -> Option<&ScreenId> {
        if let Some(overlay) = &self.overlay {
            Some(overlay)
        } else {
            self.stack.last()
        }
    }

    pub fn push(&mut self, screen_id: ScreenId) -> AppResult<()> {
        if self.stack.len() >= self.max_depth {
            self.stack.remove(0);
        }

        self.overlay.take();
        self.stack.push(screen_id);

        Ok(())
    }

    pub fn push_overlay(&mut self, id: ScreenId) -> AppResult<()> {
        self.overlay = Some(id);

        Ok(())
    }

    pub fn pop(&mut self) -> Option<ScreenId> {
        self.overlay.take();

        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }
}
