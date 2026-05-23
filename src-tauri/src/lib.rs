pub mod auth;
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
use tauri_plugin_opener::OpenerExt;

use cli::{Cli, Command};
use core::usage_store::UsageStore;
use settings::{
    get_provider_catalog, get_provider_credential_status, get_settings, save_settings,
    SettingsState,
};
use tray::{
    maybe_show_main_for_dev, open_app_window, set_tray_panel_height, setup_app_windows,
    setup_main_panel, setup_tray, show_main_panel, sync_tray_usage,
};
use widget::{hide_widget, setup_widget, show_widget, toggle_widget};

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return run_cli(command);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            app.manage(SettingsState::new(app.handle())?);
            app.manage(UsageStore::new(None));
            setup_main_panel(app.handle())?;
            setup_app_windows(app.handle())?;
            setup_tray(app.handle())?;
            setup_widget(app.handle())?;
            maybe_show_main_for_dev(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            quit_app,
            get_settings,
            save_settings,
            get_provider_catalog,
            get_provider_credential_status,
            status::get_usage_snapshots,
            status::refresh_provider,
            show_main_panel,
            open_app_window,
            open_external_url,
            set_tray_panel_height,
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
