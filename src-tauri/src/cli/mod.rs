use clap::{Parser, Subcommand};

pub mod usage;

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
    },
    Cost {
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(long, default_value_t = 30)]
        days: u16,
    },
    Config {
        key: Option<String>,
        value: Option<String>,
    },
    Update {
        action: String,
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
