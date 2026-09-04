use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

/// RAII guard for fullscreen TUI mode.
///
/// [`TuiGuard::enter`] enables raw mode and switches to the alternate screen;
/// dropping the guard restores both, so early returns and `?` unwinding cannot
/// leave the user's terminal in a broken state.
pub struct TuiGuard;

impl TuiGuard {
    /// Enter raw mode + alternate screen. Errors when there is no terminal
    /// (piped output, CI), which is how callers detect non-interactive use.
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen)?;
        Ok(Self)
    }

    fn restore() {
        let mut out = io::stdout();
        let _ = execute!(out, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

impl Drop for TuiGuard {
    fn drop(&mut self) {
        Self::restore();
    }
}

/// Install a panic hook that restores the terminal before delegating to the
/// previous hook, so panics inside TUI mode do not corrupt the shell session.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        TuiGuard::restore();
        previous(info);
    }));
}
