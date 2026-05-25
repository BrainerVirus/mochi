use tauri::{ActivationPolicy, AppHandle, Manager};

/// Hide the app from the Dock and Cmd+Tab while only the menu bar tray is active.
pub fn set_tray_only_activation_policy(app: &AppHandle) {
    let _ = app.set_activation_policy(ActivationPolicy::Accessory);
}

/// Show the app in the Dock while a dedicated settings/about window is visible.
pub fn set_regular_activation_policy(app: &AppHandle) {
    let _ = app.set_activation_policy(ActivationPolicy::Regular);
    ensure_dock_icon();
}

/// Applies the MochiChibi dock icon when the app enters the Dock.
///
/// Tauri sets a dev icon on launch, but accessory apps are not in the Dock yet.
/// Re-applying after switching to regular activation avoids the generic `exec` glyph.
pub fn ensure_dock_icon() {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSString;

    let icon_path = format!("{}/icons/icon.icns", env!("CARGO_MANIFEST_DIR"));
    let path = NSString::from_str(&icon_path);

    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let app = NSApplication::sharedApplication(mtm);

    unsafe {
        let Some(icon) = NSImage::initWithContentsOfFile(NSImage::alloc(), &path) else {
            eprintln!("[mochi] dock icon unavailable at {icon_path}");
            return;
        };
        app.setApplicationIconImage(Some(&icon));
    }
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
