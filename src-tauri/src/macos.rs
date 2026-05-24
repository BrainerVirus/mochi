use tauri::{ActivationPolicy, AppHandle, Manager};

/// Hide the app from the Dock and Cmd+Tab while only the menu bar tray is active.
pub fn set_tray_only_activation_policy(app: &AppHandle) {
    let _ = app.set_activation_policy(ActivationPolicy::Accessory);
}

/// Show the app in the Dock while a dedicated settings/about window is visible.
pub fn set_regular_activation_policy(app: &AppHandle) {
    let _ = app.set_activation_policy(ActivationPolicy::Regular);
}

const APP_WINDOW_LABEL: &str = "settings";

/// Re-apply tray-only policy when no dedicated app windows remain visible.
pub fn sync_activation_policy_for_visible_windows(app: &AppHandle) {
    let settings_visible = app
        .get_webview_window(APP_WINDOW_LABEL)
        .is_some_and(|window| window.is_visible().unwrap_or(false));

    if settings_visible {
        set_regular_activation_policy(app);
    } else {
        set_tray_only_activation_policy(app);
    }
}
