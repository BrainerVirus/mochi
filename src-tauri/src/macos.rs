use std::path::Path;

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

/// Restores the Dock icon when the app enters the Dock.
///
/// - **Bundled `.app`**: resolve via Icon Services (`NSWorkspace.iconForFile`) so macOS applies the squircle.
/// - **Dev binary**: re-apply the bundled PNG from `src-tauri/icons/` (never clear to `None`).
pub fn ensure_dock_icon() {
    if let Some(bundle_path) = app_bundle_path() {
        apply_bundled_dock_icon(&bundle_path);
        return;
    }

    apply_dev_dock_icon();
}

fn apply_bundled_dock_icon(bundle_path: &str) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSWorkspace};
    use objc2_foundation::NSString;

    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let app = NSApplication::sharedApplication(mtm);

    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let path = NSString::from_str(bundle_path);
        let icon = workspace.iconForFile(&path);
        app.setApplicationIconImage(Some(&icon));
    }
}

fn apply_dev_dock_icon() {
    use objc2::AllocAnyThread;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let png_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("icons/128x128@2x.png");
    let Ok(bytes) = std::fs::read(&png_path) else {
        return;
    };

    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let app = NSApplication::sharedApplication(mtm);

    unsafe {
        let data = NSData::with_bytes(&bytes);
        let icon = NSImage::initWithData(NSImage::alloc(), &data);
        if icon.is_none() {
            return;
        }
        app.setApplicationIconImage(icon.as_deref());
    }
}

/// Returns the `.app` bundle path when running inside a macOS app bundle.
fn app_bundle_path() -> Option<String> {
    app_bundle_path_from_exe(&std::env::current_exe().ok()?)
}

fn app_bundle_path_from_exe(exe: &Path) -> Option<String> {
    // …/Mochi.app/Contents/MacOS/mochi
    let macos_dir = exe.parent()?;
    let contents_dir = macos_dir.parent()?;
    if contents_dir.file_name()?.to_str()? != "Contents" {
        return None;
    }
    let app_bundle = contents_dir.parent()?;
    if app_bundle.extension()?.to_str()? != "app" {
        return None;
    }
    Some(app_bundle.to_string_lossy().into_owned())
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

#[cfg(test)]
mod tests {
    use super::app_bundle_path_from_exe;
    use std::path::Path;

    #[test]
    fn app_bundle_path_is_none_outside_app_bundle() {
        assert!(app_bundle_path_from_exe(Path::new("/tmp/mochi")).is_none());
    }

    #[test]
    fn app_bundle_path_resolves_from_macos_executable() {
        let exe = Path::new("/Applications/Mochi.app/Contents/MacOS/mochi");
        assert_eq!(
            app_bundle_path_from_exe(exe).as_deref(),
            Some("/Applications/Mochi.app")
        );
    }
}
