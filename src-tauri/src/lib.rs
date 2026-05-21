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
use tauri::Manager;

use cli::{Cli, Command};
use settings::{get_settings, save_settings, SettingsState};
use tray::{setup_tray, show_main_panel, sync_tray_usage};
use widget::{hide_widget, setup_widget, show_widget, toggle_widget};

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
        .setup(|app| {
            app.manage(SettingsState::new(app.handle())?);
            setup_tray(app.handle())?;
            setup_widget(app.handle())?;
            maybe_show_main_for_dev(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            get_settings,
            save_settings,
            status::get_usage_snapshots,
            status::refresh_provider,
            show_main_panel,
            sync_tray_usage,
            updater::check_for_update,
            updater::install_update,
            show_widget,
            hide_widget,
            toggle_widget
        ])
        .run(tauri::generate_context!())
        .map_err(|error| anyhow::anyhow!(error))
}

/// When `MOCHI_DEV_SHOW_MAIN=1`, opens the main window at startup so devs can use DevTools
/// without locating the menu bar icon (common on 14" MacBooks with menu bar overflow).
fn maybe_show_main_for_dev(app: &tauri::AppHandle) {
    #[cfg(not(debug_assertions))]
    let _ = app;

    #[cfg(debug_assertions)]
    {
        let show = std::env::var_os("MOCHI_DEV_SHOW_MAIN").is_some_and(|value| {
            value != "0" && value != "false"
        });
        if show {
            tray::show_main_window(app, "/");
            eprintln!("[mochi] MOCHI_DEV_SHOW_MAIN: opened main window for dev validation");
        }
    }
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
