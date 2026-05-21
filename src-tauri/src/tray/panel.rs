use std::sync::Mutex;

use tauri::{
    AppHandle, Emitter, Manager, Runtime, WebviewWindow, WindowEvent,
    tray::TrayIconEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};

pub const MAIN_PANEL_LABEL: &str = "main";
pub const SETTINGS_WINDOW_LABEL: &str = "settings";

/// Matches `src/lib/utils/tray-panel-layout.ts` and `tauri.conf.json` main window width.
pub const TRAY_PANEL_WIDTH: f64 = 360.0;

/// Matches `TRAY_PANEL_MIN_HEIGHT_PX` in the frontend layout module.
pub const TRAY_PANEL_MIN_HEIGHT: f64 = 160.0;

/// Default upper bound when the frontend cannot read screen height (480 + margin).
#[allow(dead_code)]
pub const TRAY_PANEL_DEFAULT_MAX_HEIGHT: f64 = 496.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrayIconRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Default)]
pub struct TrayIconRectState(pub Mutex<Option<TrayIconRect>>);

/// Returns whether `MOCHI_DEV_SHOW_MAIN` requests opening the panel at startup.
pub fn dev_show_main_enabled() -> bool {
    std::env::var_os("MOCHI_DEV_SHOW_MAIN").is_some_and(|value| value != "0" && value != "false")
}

pub fn setup_main_panel(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(TrayIconRectState::default());

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
    // Keeps the popover visible when switching spaces on macOS.
    #[cfg(target_os = "macos")]
    let _ = window.set_visible_on_all_workspaces(true);
    let _ = window.hide();
    Ok(())
}

/// Updates positioner plugin state and caches the latest tray icon bounds.
pub fn record_tray_icon_event<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    tauri_plugin_positioner::on_tray_event(app, event);

    if let Some(rect) = tray_icon_rect_from_event(event) {
        if let Some(state) = app.try_state::<TrayIconRectState>() {
            *state.0.lock().expect("tray icon rect lock poisoned") = Some(rect);
        }
    }
}

pub fn tray_icon_rect_from_event(event: &TrayIconEvent) -> Option<TrayIconRect> {
    let rect = match event {
        TrayIconEvent::Click { rect, .. }
        | TrayIconEvent::Enter { rect, .. }
        | TrayIconEvent::Leave { rect, .. }
        | TrayIconEvent::Move { rect, .. } => rect,
        _ => return None,
    };

    let size = rect.size.to_physical(1.0);
    let position: tauri::PhysicalPosition<f64> = rect.position.to_physical(1.0);

    Some(tray_icon_rect_from_physical(position, size))
}

pub fn tray_icon_rect_from_physical(
    position: tauri::PhysicalPosition<f64>,
    size: tauri::PhysicalSize<f64>,
) -> TrayIconRect {
    TrayIconRect {
        x: position.x as i32,
        y: position.y as i32,
        width: size.width as i32,
        height: size.height as i32,
    }
}

/// Primary anchor for the tray popover relative to the system tray icon.
///
/// macOS menu bar: open below the icon with left edges aligned.
/// Windows and Linux: `TrayLeft` uses the positioner plugin's taskbar-aware logic
/// (panel above the icon when the taskbar is bottom-docked, below when top-docked).
pub fn tray_panel_anchor_position() -> Position {
    #[cfg(target_os = "macos")]
    {
        Position::TrayBottomLeft
    }

    #[cfg(not(target_os = "macos"))]
    {
        Position::TrayLeft
    }
}

/// Screen-corner fallback when tray icon geometry is unavailable.
pub fn tray_panel_fallback_position() -> Position {
    #[cfg(target_os = "macos")]
    {
        Position::TopRight
    }

    #[cfg(not(target_os = "macos"))]
    {
        Position::BottomRight
    }
}

pub fn has_cached_tray_icon_rect(app: &AppHandle) -> bool {
    app.try_state::<TrayIconRectState>().is_some_and(|state| {
        state
            .0
            .lock()
            .ok()
            .is_some_and(|cached| cached.is_some())
    })
}

/// Resizes the tray popover to content height (logical px), reclamping anchor when visible.
#[tauri::command]
pub fn set_tray_panel_height(app: AppHandle, height: f64) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_PANEL_LABEL) else {
        return Err(format!("missing tray panel window: {MAIN_PANEL_LABEL}"));
    };

    let height = height.max(TRAY_PANEL_MIN_HEIGHT);
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: TRAY_PANEL_WIDTH,
            height,
        }))
        .map_err(|error| error.to_string())?;

    if window.is_visible().unwrap_or(false) {
        position_tray_panel(&app, &window)?;
    }

    Ok(())
}

/// Opens the tray panel window. Useful for dev validation when the tray icon is hard to reach.
#[tauri::command]
pub fn show_main_panel(app: AppHandle) {
    show_tray_panel(&app, "/");
}

/// Opens or focuses the dedicated settings/about window (not the tray popover).
#[tauri::command]
pub fn open_app_window(app: AppHandle, path: String) -> Result<(), String> {
    if let Some(tray_panel) = app.get_webview_window(MAIN_PANEL_LABEL) {
        let _ = tray_panel.hide();
    }

    let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) else {
        return Err(format!("missing app window: {SETTINGS_WINDOW_LABEL}"));
    };

    app.emit("app-navigate", path.as_str())
        .map_err(|error| error.to_string())?;

    if window.is_visible().unwrap_or(false) {
        window.set_focus().map_err(|error| error.to_string())?;
    } else {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
    }

    Ok(())
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

    open_visible_tray_panel(app, &window);
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

    open_visible_tray_panel(app, &window);
}

fn open_visible_tray_panel(app: &AppHandle, window: &WebviewWindow) {
    let _ = position_tray_panel(app, window);
    let _ = window.show();
    let _ = window.set_focus();
}

fn position_tray_panel(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    if has_cached_tray_icon_rect(app) {
        window
            .move_window_constrained(tray_panel_anchor_position())
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    window
        .move_window(tray_panel_fallback_position())
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
            "[mochi] MOCHI_DEV_SHOW_MAIN: opened tray panel centered (debug helper — use tray icon for anchored popover)"
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

    #[test]
    fn tray_icon_rect_converts_physical_bounds() {
        use tauri::PhysicalPosition;
        use tauri::PhysicalSize;

        assert_eq!(
            tray_icon_rect_from_physical(
                PhysicalPosition::new(120.0, 8.0),
                PhysicalSize::new(22.0, 22.0)
            ),
            TrayIconRect {
                x: 120,
                y: 8,
                width: 22,
                height: 22,
            }
        );
    }

    #[test]
    fn tray_panel_fallback_uses_platform_corner() {
        #[cfg(target_os = "macos")]
        assert!(matches!(
            tray_panel_fallback_position(),
            Position::TopRight
        ));

        #[cfg(not(target_os = "macos"))]
        assert!(matches!(
            tray_panel_fallback_position(),
            Position::BottomRight
        ));
    }

    #[test]
    fn tray_panel_anchor_uses_platform_tray_position() {
        #[cfg(target_os = "macos")]
        assert!(matches!(
            tray_panel_anchor_position(),
            Position::TrayBottomLeft
        ));

        #[cfg(not(target_os = "macos"))]
        assert!(matches!(tray_panel_anchor_position(), Position::TrayLeft));
    }

    #[test]
    fn tray_panel_height_bounds_match_frontend_layout() {
        assert_eq!(TRAY_PANEL_WIDTH, 360.0);
        assert_eq!(TRAY_PANEL_MIN_HEIGHT, 160.0);
        assert_eq!(TRAY_PANEL_DEFAULT_MAX_HEIGHT, 496.0);
    }
}
