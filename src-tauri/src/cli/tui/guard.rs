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
        Self::enter_with(&mut io::stdout(), enable_raw_mode, disable_raw_mode)
    }

    fn enter_with<W: io::Write>(
        writer: &mut W,
        enable: impl FnOnce() -> io::Result<()>,
        disable: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<Self> {
        enable()?;
        if let Err(error) = execute!(writer, EnterAlternateScreen) {
            let _ = disable();
            return Err(error);
        }
        Ok(Self)
    }

    fn restore() {
        Self::restore_with(&mut io::stdout(), disable_raw_mode);
    }

    fn restore_with<W: io::Write>(writer: &mut W, disable: impl FnOnce() -> io::Result<()>) {
        let _ = execute!(writer, LeaveAlternateScreen);
        let _ = disable();
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
        let disabled = RefCell::new(false);
        let mut writer = FailingWriter;
        let result = TuiGuard::enter_with(
            &mut writer,
            || Ok(()),
            || {
                *disabled.borrow_mut() = true;
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

    #[test]
    fn restore_leaves_alt_screen_before_disabling_raw_mode() {
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::default();
        let mut writer = OrderWriter {
            buf: Vec::new(),
            log: Rc::clone(&log),
        };
        let log_in_hook = Rc::clone(&log);
        TuiGuard::restore_with(&mut writer, || {
            log_in_hook.borrow_mut().push("disable");
            Ok(())
        });
        assert_eq!(log.borrow().as_slice(), ["leave-write", "disable"]);
        assert!(
            writer.buf.windows(8).any(|w| w == b"\x1b[?1049l"),
            "restore must emit the leave-alternate-screen sequence"
        );
    }
}
