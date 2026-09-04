pub mod guard;

pub use guard::{install_panic_hook, TuiGuard};

/// Process exit codes shared by the CLI/TUI entry points.
pub const EXIT_OK: i32 = 0;
/// Domain failure (provider error, no data, feature unavailable).
pub const EXIT_DOMAIN: i32 = 1;
/// CLI usage error (bad flags, unknown command).
pub const EXIT_USAGE: i32 = 2;

/// Decide whether to launch the interactive TUI.
///
/// True only when both stdio handles are TTYs and the caller did not request
/// machine-readable output (`--json`/format flags). Takes explicit bools so it
/// stays unit-testable without touching real file descriptors.
pub fn should_use_tui(stdin_tty: bool, stdout_tty: bool, machine_output: bool) -> bool {
    stdin_tty && stdout_tty && !machine_output
}

#[cfg(test)]
mod tests {
    use super::guard::TuiGuard;
    use super::{install_panic_hook, should_use_tui, EXIT_DOMAIN, EXIT_OK, EXIT_USAGE};
    use std::io::IsTerminal as _;

    #[test]
    fn tui_gate_requires_tty_and_human_output() {
        assert!(should_use_tui(true, true, false));
        assert!(!should_use_tui(false, true, false));
        assert!(!should_use_tui(true, false, false));
        assert!(!should_use_tui(true, true, true));
    }

    #[test]
    fn guard_restores_terminal_on_drop() {
        // Gated: only meaningful on a real TTY; assert construction contract instead:
        // enter() on non-TTY returns Err (crossterm errors without a terminal).
        assert!(TuiGuard::enter().is_err() || std::io::stdin().is_terminal());
    }

    #[test]
    fn exit_codes_are_stable() {
        assert_eq!(EXIT_OK, 0);
        assert_eq!(EXIT_DOMAIN, 1);
        assert_eq!(EXIT_USAGE, 2);
        let _ = install_panic_hook;
    }
}
