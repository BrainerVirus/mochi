use tauri::{AppHandle, WebviewWindow};

/// Taskbar / window icon for dedicated app windows (settings, about, update).
pub fn sync_app_window_branding(app: &AppHandle, window: &WebviewWindow) {
    if let Some(icon) = app.default_window_icon() {
        let _ = window.set_icon(icon.clone());
    }

    #[cfg(target_os = "macos")]
    crate::macos::ensure_dock_icon();
}
