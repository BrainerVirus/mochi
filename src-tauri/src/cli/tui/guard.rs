use std::io;

use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

/// RAII guard for fullscreen TUI mode.
///
/// [`TuiGuard::enter`] enables raw mode, switches to the alternate screen,
/// and enables bracketed paste (so terminal pastes arrive as a single
/// `Event::Paste` instead of per-char key events); dropping the guard
/// restores all three, so early returns and `?` unwinding cannot
/// leave the user's terminal in a broken state.
pub struct TuiGuard<W: io::Write = io::Stdout> {
    writer: W,
    disable: Option<Box<dyn FnOnce() -> io::Result<()>>>,
}

impl TuiGuard<io::Stdout> {
    /// Enter raw mode + alternate screen. Errors when there is no terminal
    /// (piped output, CI), which is how callers detect non-interactive use.
    pub fn enter() -> io::Result<Self> {
        Self::enter_with(io::stdout(), enable_raw_mode, disable_raw_mode)
    }

    fn restore() {
        Self::restore_with(&mut io::stdout(), disable_raw_mode);
    }
}

impl<W: io::Write> TuiGuard<W> {
    fn enter_with(
        mut writer: W,
        enable: impl FnOnce() -> io::Result<()>,
        disable: impl FnOnce() -> io::Result<()> + 'static,
    ) -> io::Result<Self> {
        enable()?;
        if let Err(error) = execute!(writer, EnterAlternateScreen) {
            let _ = disable();
            return Err(error);
        }
        if let Err(error) = execute!(writer, EnableBracketedPaste) {
            Self::restore_with(&mut writer, disable);
            return Err(error);
        }
        Ok(Self {
            writer,
            disable: Some(Box::new(disable)),
        })
    }

    fn restore_with(writer: &mut W, disable: impl FnOnce() -> io::Result<()>) {
        let _ = execute!(writer, LeaveAlternateScreen);
        let _ = execute!(writer, DisableBracketedPaste);
        let _ = disable();
    }
}

impl<W: io::Write> Drop for TuiGuard<W> {
    fn drop(&mut self) {
        if let Some(disable) = self.disable.take() {
            Self::restore_with(&mut self.writer, disable);
        }
    }
}

/// Install a panic hook that restores the terminal before delegating to the
/// previous hook, so panics inside TUI mode do not corrupt the shell session.
///
/// NOTE: when Tasks 7-9 wire this into the CLI entry point, install it LAST
/// so it wraps (and delegates to) the diagnostics hook — hook order is LIFO,
/// and the terminal must be restored before the diagnostics logger runs.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        TuiGuard::restore();
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::TuiGuard;
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;

    struct FailingWriter;

    impl io::Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("no screen"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn enter_disables_raw_mode_when_alt_screen_fails() {
        let disabled = Rc::new(RefCell::new(false));
        let writer = FailingWriter;
        let disabled_in_hook = Rc::clone(&disabled);
        let result = TuiGuard::enter_with(
            writer,
            || Ok(()),
            move || {
                *disabled_in_hook.borrow_mut() = true;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(
            *disabled.borrow(),
            "raw mode must be disabled when entering the alternate screen fails"
        );
    }

    struct OrderWriter {
        buf: Vec<u8>,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl io::Write for OrderWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buf.extend_from_slice(buf);
            self.log.borrow_mut().push("leave-write");
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct SharedWriter {
        buf: Rc<RefCell<Vec<u8>>>,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl io::Write for SharedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buf.borrow_mut().extend_from_slice(buf);
            self.log.borrow_mut().push("leave-write");
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn dropping_guard_leaves_alt_screen_before_disabling_raw_mode() {
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::default();
        let buf: Rc<RefCell<Vec<u8>>> = Rc::default();
        let writer = SharedWriter {
            buf: Rc::clone(&buf),
            log: Rc::clone(&log),
        };
        let log_in_hook = Rc::clone(&log);
        let guard = TuiGuard::enter_with(
            writer,
            || Ok(()),
            move || {
                log_in_hook.borrow_mut().push("disable");
                Ok(())
            },
        );
        assert!(guard.is_ok(), "enter with a working writer must succeed");
        // Ignore the enter-phase write; the drop below must emit the leave
        // and disable-paste sequences first and disable raw mode second.
        log.borrow_mut().clear();
        buf.borrow_mut().clear();
        drop(guard);
        assert_eq!(
            log.borrow().as_slice(),
            ["leave-write", "leave-write", "disable"]
        );
        assert!(
            buf.borrow().windows(8).any(|w| w == b"\x1b[?1049l"),
            "dropping the guard must emit the leave-alternate-screen sequence"
        );
        assert!(
            buf.borrow().windows(8).any(|w| w == b"\x1b[?2004l"),
            "dropping the guard must emit the disable-bracketed-paste sequence"
        );
    }

    #[test]
    fn guard_enables_and_disables_bracketed_paste() {
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::default();
        let buf: Rc<RefCell<Vec<u8>>> = Rc::default();
        let writer = SharedWriter {
            buf: Rc::clone(&buf),
            log: Rc::clone(&log),
        };
        let log_in_hook = Rc::clone(&log);
        let guard = TuiGuard::enter_with(
            writer,
            || Ok(()),
            move || {
                log_in_hook.borrow_mut().push("disable");
                Ok(())
            },
        );
        assert!(guard.is_ok(), "enter with a working writer must succeed");
        assert!(
            buf.borrow().windows(8).any(|w| w == b"\x1b[?2004h"),
            "entering the guard must enable bracketed paste"
        );
        buf.borrow_mut().clear();
        drop(guard);
        assert!(
            buf.borrow().windows(8).any(|w| w == b"\x1b[?2004l"),
            "dropping the guard must disable bracketed paste"
        );
    }

    #[test]
    fn restore_leaves_alt_screen_before_disabling_raw_mode() {
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::default();
        let mut writer = OrderWriter {
            buf: Vec::new(),
            log: Rc::clone(&log),
        };
        let log_in_hook = Rc::clone(&log);
        TuiGuard::<OrderWriter>::restore_with(&mut writer, || {
            log_in_hook.borrow_mut().push("disable");
            Ok(())
        });
        assert_eq!(
            log.borrow().as_slice(),
            ["leave-write", "leave-write", "disable"]
        );
        assert!(
            writer.buf.windows(8).any(|w| w == b"\x1b[?1049l"),
            "restore must emit the leave-alternate-screen sequence"
        );
        assert!(
            writer.buf.windows(8).any(|w| w == b"\x1b[?2004l"),
            "restore must emit the disable-bracketed-paste sequence"
        );
    }
}
