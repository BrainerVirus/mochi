use tauri::{AppHandle, WebviewWindow};

/// Taskbar / window icon for dedicated app windows (settings, about, update).
pub fn sync_app_window_branding(app: &AppHandle, window: &WebviewWindow) {
    if let Some(icon) = app.default_window_icon() {
        let _ = window.set_icon(icon.clone());
    }

    #[cfg(target_os = "macos")]
    crate::macos::ensure_dock_icon();
}

#[cfg(test)]
mod tests {
    // The branding helper requires a live Tauri AppHandle/WebviewWindow, so
    // we can't exercise the real call paths in a unit test. We document
    // the intent here for future integration coverage.
    #[test]
    fn branding_module_compiles() {
        // intentionally no-op
    }
}
