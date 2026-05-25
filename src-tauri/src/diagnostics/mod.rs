mod log;
mod report;
mod state;

pub use log::{init, log_line};
pub use report::{build_bundle_path, build_summary, run_cli_diagnostics};
pub use state::{log_visible_windows, DiagnosticsState};

use serde::Deserialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendBootPayload {
    pub window_label: String,
    pub location_href: String,
    pub pathname: String,
    pub user_agent: String,
    pub has_tauri_internals: bool,
    pub target_route: String,
    pub tauri_label: Option<String>,
    pub tauri_label_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendErrorPayload {
    pub window_label: Option<String>,
    pub message: String,
    pub source: Option<String>,
    pub stack: Option<String>,
}

pub fn setup(app: &AppHandle) -> Result<(), String> {
    init(app)?;
    log_line("diagnostics", "initialized");
    app.manage(DiagnosticsState::new());
    Ok(())
}

#[tauri::command]
pub fn report_frontend_boot(
    state: tauri::State<'_, DiagnosticsState>,
    payload: FrontendBootPayload,
) -> Result<(), String> {
    state.record_boot(payload);
    Ok(())
}

#[tauri::command]
pub fn report_frontend_error(
    state: tauri::State<'_, DiagnosticsState>,
    payload: FrontendErrorPayload,
) -> Result<(), String> {
    state.record_frontend_error(payload);
    Ok(())
}

#[tauri::command]
pub fn get_diagnostics_summary(app: tauri::AppHandle) -> Result<String, String> {
    build_summary(
        &app,
        app.try_state::<DiagnosticsState>().as_deref(),
    )
}
