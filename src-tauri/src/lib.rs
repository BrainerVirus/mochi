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
pub mod window_policy;

use clap::Parser;
use std::path::PathBuf;
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
    get_provider_catalog, get_provider_credential_status, get_settings, load_settings,
    save_settings, settings_file_path, SettingsState,
};
use tray::{
    maybe_show_main_for_dev, open_app_window, set_tray_panel_height, setup_app_windows,
    setup_main_panel, setup_tray, show_main_panel, sync_tray_update_channel, sync_tray_usage,
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
            if window_policy::should_precreate_decorated_windows_at_startup() {
                setup_app_windows(app.handle())?;
                setup_widget(app.handle())?;
            }
            setup_tray(app.handle())?;
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
            sync_tray_update_channel,
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
    let _ = repository
        .set_initial_provider_detection_completed(reconciliation.initial_detection_completed);

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
        Command::Usage {
            provider,
            refresh,
            json,
        } => {
            let states = cli_usage_states(provider.as_deref(), refresh)?;
            if json {
                println!("{}", cli::usage::format_usage_json(&states)?);
            } else {
                println!("{}", cli::usage::format_usage_text(&states));
            }
        }
        Command::StatusBar { format } => {
            let states = cli_usage_states(None, false)?;
            let output = status_bar::format_output_from_states(&format, &states);
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

fn cli_usage_states(
    provider: Option<&str>,
    refresh: bool,
) -> anyhow::Result<Vec<core::usage_state::ProviderUsageState>> {
    let settings = cli_config_dir()
        .map(|dir| load_settings(&settings_file_path(&dir)))
        .unwrap_or_default();
    let repository = cli_data_dir()
        .map(|dir| dir.join("usage.sqlite3"))
        .and_then(|path| core::usage_repository::SqliteUsageRepository::open(&path).ok());
    let store = repository
        .map(|repository| UsageStore::with_repository(std::sync::Arc::new(repository)))
        .unwrap_or_else(|| UsageStore::new(None));

    let mut enabled = settings.enabled_providers.clone();
    if let Some(provider) = provider {
        let provider = core::models::ProviderId::parse(provider)
            .ok_or_else(|| anyhow::anyhow!("unknown provider: {provider}"))?;
        enabled = vec![provider.as_str().to_string()];
    }

    let mut settings = settings;
    settings.enabled_providers = enabled;
    let _ = store.load_latest_states(&settings.enabled_providers);

    if refresh {
        eprintln!("Refreshing usage...");
        let runtime = tokio::runtime::Runtime::new()?;
        let _ = runtime.block_on(status::refresh_enabled_snapshots(&store, &settings));
        let _ = store.load_latest_states(&settings.enabled_providers);
    }

    Ok(status::read_cached_usage_states(&store, &settings))
}

fn cli_config_dir() -> Option<PathBuf> {
    platform_base_config_dir().map(|base| base.join("app.mochi.Mochi"))
}

fn cli_data_dir() -> Option<PathBuf> {
    platform_base_data_dir().map(|base| base.join("app.mochi.Mochi"))
}

#[cfg(target_os = "macos")]
fn platform_base_config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"))
}

#[cfg(target_os = "macos")]
fn platform_base_data_dir() -> Option<PathBuf> {
    platform_base_config_dir()
}

#[cfg(target_os = "windows")]
fn platform_base_config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "windows")]
fn platform_base_data_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_base_config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_base_data_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
}
