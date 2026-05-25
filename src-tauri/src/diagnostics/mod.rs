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
    report::log_runtime_environment();
    app.manage(DiagnosticsState::new());
    Ok(())
}

pub fn log_window_action_result<E: std::fmt::Display>(
    label: &str,
    action: &str,
    result: Result<(), E>,
) {
    log_line(
        "window.action",
        &window_action_detail(label, action, result),
    );
}

pub fn window_action_detail<E: std::fmt::Display>(
    label: &str,
    action: &str,
    result: Result<(), E>,
) -> String {
    match result {
        Ok(()) => format!("{label}: {action} -> ok"),
        Err(error) => format!("{label}: {action} -> error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::window_action_detail;

    #[test]
    fn window_action_detail_formats_success_and_error() {
        assert_eq!(
            window_action_detail("settings", "hide", Ok::<(), &str>(())),
            "settings: hide -> ok"
        );
        assert_eq!(
            window_action_detail("widget", "set_focus", Err("not ready")),
            "widget: set_focus -> error: not ready"
        );
    }
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
    build_summary(&app, app.try_state::<DiagnosticsState>().as_deref())
}
