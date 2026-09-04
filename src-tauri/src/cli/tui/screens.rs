//! Shared screen identity + list navigation for the CLI/TUI entry points.

/// Fullscreen TUI screens. Tasks 8-9 add their variants here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    ConfigWizard,
}

/// Shared up/down cursor over a list of `len` rows. Saturates at the ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListNav {
    selected: usize,
    len: usize,
}

impl ListNav {
    pub fn new(len: usize) -> Self {
        Self { selected: 0, len }
    }

    pub fn selected(&self) -> usize {
        self.selected.min(self.len.saturating_sub(1))
    }

    pub fn set_len(&mut self, len: usize) {
        self.len = len;
        self.selected = self.selected.min(len.saturating_sub(1));
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.len > 0 {
            self.selected = (self.selected + 1).min(self.len - 1);
        }
    }
}
