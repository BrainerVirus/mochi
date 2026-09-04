pub mod config_wizard;
pub mod cost_view;
pub mod guard;
pub mod screens;
pub mod update_flow;
pub mod usage_dashboard;

pub use guard::{install_panic_hook, TuiGuard};

/// Process exit codes shared by the CLI/TUI entry points.
pub const EXIT_OK: i32 = 0;
/// Domain failure (provider error, no data, feature unavailable).
pub const EXIT_DOMAIN: i32 = 1;
/// CLI usage error (bad flags, unknown command).
pub const EXIT_USAGE: i32 = 2;

/// Decide whether to launch the interactive TUI.
///
/// True only when both stdio handles are TTYs, the caller did not request
/// machine-readable output (`--json`/format flags), and `TERM` is not `dumb`.
/// Takes explicit bools so it stays unit-testable without touching real file
/// descriptors or environment variables; use [`should_use_tui_env`] for the
/// thin env-reading caller.
pub fn should_use_tui(
    stdin_tty: bool,
    stdout_tty: bool,
    machine_output: bool,
    dumb_term: bool,
) -> bool {
    stdin_tty && stdout_tty && !machine_output && !dumb_term
}

/// Env-reading wrapper around [`should_use_tui`]: treats `TERM=dumb` as a
/// non-interactive terminal. A missing `TERM` is not dumb.
pub fn should_use_tui_env(stdin_tty: bool, stdout_tty: bool, machine_output: bool) -> bool {
    let dumb_term = std::env::var("TERM")
        .map(|term| term == "dumb")
        .unwrap_or(false);
    should_use_tui(stdin_tty, stdout_tty, machine_output, dumb_term)
}

#[cfg(test)]
mod tests {
    use super::{
        install_panic_hook, should_use_tui, should_use_tui_env, EXIT_DOMAIN, EXIT_OK, EXIT_USAGE,
    };

    #[test]
    fn tui_gate_requires_tty_and_human_output() {
        assert!(should_use_tui(true, true, false, false));
        assert!(!should_use_tui(false, true, false, false));
        assert!(!should_use_tui(true, false, false, false));
        assert!(!should_use_tui(true, true, true, false));
    }

    #[test]
    fn tui_gate_rejects_dumb_terminal() {
        assert!(!should_use_tui(true, true, false, true));
    }

    #[test]
    fn tui_gate_env_rejects_dumb_term() {
        let saved = std::env::var("TERM").ok();
        std::env::set_var("TERM", "dumb");
        assert!(!should_use_tui_env(true, true, false));
        std::env::set_var("TERM", "xterm-256color");
        assert!(should_use_tui_env(true, true, false));
        if let Some(value) = saved {
            std::env::set_var("TERM", value);
        } else {
            std::env::remove_var("TERM");
        }
    }

    #[test]
    fn exit_codes_are_stable() {
        assert_eq!(EXIT_OK, 0);
        assert_eq!(EXIT_DOMAIN, 1);
        assert_eq!(EXIT_USAGE, 2);
        // Installing twice is harmless: each install wraps the previous hook.
        install_panic_hook();
        install_panic_hook();
    }
}
