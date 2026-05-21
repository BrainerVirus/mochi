pub mod cli;
pub mod core;
pub mod providers;
pub mod settings;
pub mod status;
pub mod status_bar;
pub mod tray;
pub mod updater;
pub mod widget;

use clap::Parser;

use cli::{Cli, Command};

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return run_cli(command);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            app_version,
            status::get_usage_snapshots,
            updater::check_for_update,
            updater::install_update
        ])
        .run(tauri::generate_context!())
        .map_err(|error| anyhow::anyhow!(error))
}

fn run_cli(command: Command) -> anyhow::Result<()> {
    match command {
        Command::StatusBar { format } => {
            let output = status_bar::format_output(&format, 42, "Claude");
            println!("{output}");
        }
        _ => {
            eprintln!("CLI subcommand not yet implemented: {command:?}");
            std::process::exit(2);
        }
    }

    Ok(())
}
