use tauri::{
    AppHandle, Emitter, Manager, Runtime, WebviewWindow, WindowEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};

pub const MAIN_PANEL_LABEL: &str = "main";

/// Returns whether `MOCHI_DEV_SHOW_MAIN` requests opening the panel at startup.
pub fn dev_show_main_enabled() -> bool {
    std::env::var_os("MOCHI_DEV_SHOW_MAIN").is_some_and(|value| value != "0" && value != "false")
}

pub fn setup_main_panel(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let Some(window) = app.get_webview_window(MAIN_PANEL_LABEL) else {
        return Ok(());
    };

    prepare_main_panel_window(&window)?;

    let window_for_events = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            let _ = window_for_events.hide();
        }
    });

    Ok(())
}

pub fn prepare_main_panel_window(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    let _ = window.set_decorations(false);
    let _ = window.set_resizable(false);
    let _ = window.set_always_on_top(true);
    let _ = window.set_skip_taskbar(true);
    #[cfg(target_os = "macos")]
    let _ = window.set_visible_on_all_workspaces(true);
    let _ = window.hide();
    Ok(())
}

/// Opens the tray panel window. Useful for dev validation when the menu bar icon is hidden.
#[tauri::command]
pub fn show_main_panel(app: AppHandle) {
    show_tray_panel(&app, "/");
}

pub fn show_tray_panel(app: &AppHandle, path: &str) {
    let Some(window) = app.get_webview_window(MAIN_PANEL_LABEL) else {
        return;
    };

    let _ = app.emit("tray-navigate", path);

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    open_visible_tray_panel(&window);
}

/// Opens or focuses the panel without hiding when already visible (tray menu navigation).
pub fn open_tray_panel(app: &AppHandle, path: &str) {
    let Some(window) = app.get_webview_window(MAIN_PANEL_LABEL) else {
        return;
    };

    let _ = app.emit("tray-navigate", path);

    if window.is_visible().unwrap_or(false) {
        let _ = window.set_focus();
        return;
    }

    open_visible_tray_panel(&window);
}

fn open_visible_tray_panel<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = position_tray_panel(window);
    let _ = window.show();
    let _ = window.set_focus();
}

fn position_tray_panel<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    window
        .move_window_constrained(Position::TrayBottomLeft)
        .map_err(|error| error.to_string())
}

pub fn show_tray_panel_centered(app: &AppHandle, path: &str) {
    let Some(window) = app.get_webview_window(MAIN_PANEL_LABEL) else {
        return;
    };

    let _ = app.emit("tray-navigate", path);
    let _ = window.move_window(Position::Center);
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn maybe_show_main_for_dev(app: &AppHandle) {
    #[cfg(not(debug_assertions))]
    let _ = app;

    #[cfg(debug_assertions)]
    {
        if !dev_show_main_enabled() {
            return;
        }

        show_tray_panel_centered(app, "/");
        eprintln!(
            "[mochi] MOCHI_DEV_SHOW_MAIN: opened tray panel centered (no tray click — use tray icon for anchored popover)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_show_main_disabled_by_default() {
        std::env::remove_var("MOCHI_DEV_SHOW_MAIN");
        assert!(!dev_show_main_enabled());
    }

    #[test]
    fn dev_show_main_honors_env_var() {
        std::env::set_var("MOCHI_DEV_SHOW_MAIN", "1");
        assert!(dev_show_main_enabled());
        std::env::remove_var("MOCHI_DEV_SHOW_MAIN");
    }

    #[test]
    fn dev_show_main_respects_zero_and_false() {
        std::env::set_var("MOCHI_DEV_SHOW_MAIN", "0");
        assert!(!dev_show_main_enabled());
        std::env::set_var("MOCHI_DEV_SHOW_MAIN", "false");
        assert!(!dev_show_main_enabled());
        std::env::remove_var("MOCHI_DEV_SHOW_MAIN");
    }
}
