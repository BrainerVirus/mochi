pub mod app_branding;
pub mod auth;
pub mod browser;
pub mod cli;
pub mod core;
pub mod diagnostics;
pub mod frontend;
pub mod lifecycle;
pub mod linux_webkit;
pub mod linux_window_controls;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod providers;
pub mod settings;
pub mod status;
pub mod status_bar;
pub mod tray;
pub mod updater;
pub mod widget;

use clap::Parser;
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use cli::{Cli, Command};
use core::usage_repository::{SqliteUsageRepository, UsageRepository};
use core::usage_store::UsageStore;
use lifecycle::{should_prevent_exit_request, AppLifecycle};
use providers::credential_probe::detected_provider_ids;
use settings::{
    get_provider_catalog, get_provider_credential_status, get_settings, save_settings,
    SettingsState,
};
use tray::{
    maybe_show_main_for_dev, open_app_window, set_tray_panel_height, setup_app_windows,
    setup_main_panel, setup_tray, show_main_panel, sync_tray_usage,
};
use widget::{hide_widget, set_widget_height, setup_widget, show_widget, toggle_widget};

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
fn get_platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(all(unix, not(target_os = "macos"))) {
        "linux"
    } else {
        "unknown"
    }
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle, lifecycle: State<'_, AppLifecycle>) {
    lifecycle.request_quit();
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

    #[cfg(target_os = "linux")]
    linux_webkit::apply_linux_webkit_workarounds();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            macos::set_tray_only_activation_policy(app.handle());

            diagnostics::setup(app.handle())?;
            let settings_state = SettingsState::new(app.handle())?;
            let usage_store = initialize_usage_store(app.handle(), &settings_state);
            app.manage(settings_state);
            app.manage(usage_store);
            app.manage(AppLifecycle::default());
            setup_main_panel(app.handle())?;
            setup_app_windows(app.handle())?;
            setup_tray(app.handle())?;
            setup_widget(app.handle())?;
            maybe_show_main_for_dev(app.handle());
            diagnostics::log_visible_windows(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            get_platform,
            quit_app,
            get_settings,
            save_settings,
            get_provider_catalog,
            get_provider_credential_status,
            status::get_usage_snapshots,
            status::refresh_provider,
            status::refresh_enabled_providers,
            show_main_panel,
            open_app_window,
            open_external_url,
            set_tray_panel_height,
            sync_tray_usage,
            updater::check_for_update,
            updater::install_update,
            show_widget,
            hide_widget,
            set_widget_height,
            toggle_widget,
            diagnostics::report_frontend_boot,
            diagnostics::report_frontend_error,
            diagnostics::get_diagnostics_summary
        ])
        .build(tauri::generate_context!())?
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let lifecycle = app.try_state::<AppLifecycle>();
                let lifecycle = lifecycle.as_ref().map(|state| state.inner());
                if should_prevent_exit_request(lifecycle) {
                    api.prevent_exit();
                }
            }
        });

    Ok(())
}

pub fn usage_database_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("usage.sqlite3"))
        .map_err(|error| error.to_string())
}

fn initialize_usage_store(app: &tauri::AppHandle, settings_state: &SettingsState) -> UsageStore {
    let Ok(db_path) = usage_database_path(app) else {
        return UsageStore::new(None);
    };

    let Ok(repository) = SqliteUsageRepository::open(&db_path) else {
        return UsageStore::new(None);
    };

    let repository = std::sync::Arc::new(repository);
    let mut settings = match settings_state.current() {
        Ok(settings) => settings,
        Err(_) => return UsageStore::with_repository(repository),
    };
    let initial_detection_completed = repository
        .initial_provider_detection_completed()
        .unwrap_or(false);
    let detected = detected_provider_ids(&settings);
    let reconciliation = status::reconcile_first_start_enabled_providers(
        &mut settings,
        initial_detection_completed,
        detected,
    );

    if !initial_detection_completed {
        let _ = settings_state.update(settings.clone());
    }
    let _ = repository.set_initial_provider_detection_completed(
        reconciliation.initial_detection_completed,
    );

    let store = UsageStore::with_repository(repository.clone());
    let _ = store.load_latest_states(&settings.enabled_providers);
    let cutoff = (OffsetDateTime::now_utc() - Duration::days(90))
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    let _ = repository.prune_history_before(&cutoff);

    store
}

fn run_cli(command: Command) -> anyhow::Result<()> {
    match command {
        Command::StatusBar { format } => {
            let output = status_bar::format_output(&format, 42, "Claude");
            println!("{output}");
        }
        Command::Diagnostics { bundle } => {
            diagnostics::run_cli_diagnostics(bundle).map_err(|error| anyhow::anyhow!(error))?;
        }
        _ => {
            eprintln!("CLI subcommand not yet implemented: {command:?}");
            std::process::exit(2);
        }
    }

    Ok(())
}
