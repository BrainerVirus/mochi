use clap::{Parser, Subcommand};

pub mod config;
pub mod cost;
pub mod status;
pub mod tui;
pub mod update;
pub mod usage;
pub mod windows_console;

#[derive(Debug, Parser)]
#[command(name = "mochi", version, about = "Soft alerts before hard limits.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Usage {
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(long)]
        refresh: bool,
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Cost {
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(long, default_value_t = 30)]
        days: u16,
        #[arg(long)]
        json: bool,
    },
    Config {
        key: Option<String>,
        value: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Update {
        action: String,
        #[arg(long)]
        confirm: bool,
    },
    StatusBar {
        #[arg(long, default_value = "waybar")]
        format: String,
    },
    /// Print diagnostic info for support (logs, version, platform). Use when windows are blank or controls fail.
    Diagnostics {
        /// Write a redacted diagnostics bundle under the app log directory.
        #[arg(long)]
        bundle: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_cost_config_accept_json_flag() {
        assert!(matches!(
            Cli::try_parse_from(["mochi", "status", "--json"])
                .expect("status")
                .command,
            Some(Command::Status { json: true, .. })
        ));
        assert!(matches!(
            Cli::try_parse_from(["mochi", "cost", "--json"])
                .expect("cost")
                .command,
            Some(Command::Cost { json: true, .. })
        ));
        assert!(matches!(
            Cli::try_parse_from(["mochi", "config", "--json"])
                .expect("config")
                .command,
            Some(Command::Config { json: true, .. })
        ));
    }

    #[test]
    fn json_flag_implies_no_tui() {
        assert!(!tui::should_use_tui_env(true, true, true));
    }
}
