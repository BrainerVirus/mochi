use std::sync::Mutex;

use tauri::{
    tray::TrayIconEvent, AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};

#[cfg(target_os = "macos")]
use crate::macos::{set_regular_activation_policy, sync_activation_policy_for_visible_windows};
use tauri_plugin_positioner::{Position, WindowExt};

use super::window_transparency::window_uses_native_transparency;

pub const MAIN_PANEL_LABEL: &str = "main";
pub const SETTINGS_WINDOW_LABEL: &str = "settings";

/// Default settings window size (logical px).
pub const SETTINGS_WINDOW_WIDTH: f64 = 520.0;
pub const SETTINGS_WINDOW_HEIGHT: f64 = 560.0;

/// Compact about window size (logical px).
pub const ABOUT_WINDOW_WIDTH: f64 = 340.0;
pub const ABOUT_WINDOW_HEIGHT: f64 = 280.0;

/// Compact update window size (logical px).
pub const UPDATE_WINDOW_WIDTH: f64 = 340.0;
pub const UPDATE_WINDOW_HEIGHT: f64 = 280.0;

/// Matches `src/lib/utils/tray-panel-layout.ts` and `tauri.conf.json` main window width.
pub const TRAY_PANEL_WIDTH: f64 = 360.0;

/// Matches `TRAY_PANEL_MIN_HEIGHT_PX` in the frontend layout module.
pub const TRAY_PANEL_MIN_HEIGHT: f64 = 160.0;

/// Gap between panel top/bottom and screen edge (matches frontend `TRAY_PANEL_VIEWPORT_MARGIN_PX`).
pub const TRAY_PANEL_VIEWPORT_MARGIN: f64 = 16.0;

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
#[cfg(debug_assertions)]
pub fn dev_show_main_enabled() -> bool {
    std::env::var_os("MOCHI_DEV_SHOW_MAIN").is_some_and(|value| value != "0" && value != "false")
}

pub fn setup_app_windows(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let window = ensure_settings_window(app)?;
    prepare_app_window(&window)?;
    if let Err(error) = ensure_app_window_vibrancy(&window) {
        eprintln!("[mochi] app window vibrancy unavailable: {error}");
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_macos_overlay_titlebar(window: &WebviewWindow) {
    let _ = window.set_title_bar_style(tauri::TitleBarStyle::Overlay);
}

pub fn prepare_app_window(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    // Hidden at startup — keep off taskbar/dock until the user opens settings/about.
    let _ = window.set_skip_taskbar(true);

    #[cfg(target_os = "macos")]
    configure_macos_overlay_titlebar(window);

    let window_for_events = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let app = window_for_events.app_handle();
            let _ = window_for_events.set_skip_taskbar(true);
            let _ = window_for_events.hide();
            let _ = emit_tray_navigate(app, "/");
            #[cfg(target_os = "macos")]
            sync_activation_policy_for_visible_windows(app);
        }
    });

    Ok(())
}

fn emit_tray_navigate(app: &AppHandle, path: &str) -> Result<(), String> {
    app.emit_to(MAIN_PANEL_LABEL, "tray-navigate", path)
        .map_err(|error| error.to_string())
}

fn emit_app_navigate(app: &AppHandle, path: &str) -> Result<(), String> {
    app.emit_to(SETTINGS_WINDOW_LABEL, "app-navigate", path)
        .map_err(|error| error.to_string())
}

fn ensure_settings_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        return Ok(window);
    }

    let builder = WebviewWindowBuilder::new(
        app,
        SETTINGS_WINDOW_LABEL,
        WebviewUrl::App("/settings".into()),
    )
    .title("Mochi")
    .inner_size(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
    .min_inner_size(480.0, 420.0)
    .center()
    .transparent(window_uses_native_transparency())
    .decorations(true)
    .resizable(true)
    .visible(false)
    .skip_taskbar(true);

    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .accept_first_mouse(true);

    builder
        .build()
        .inspect(|window| {
            let _ = prepare_app_window(window);
            if let Err(error) = ensure_app_window_vibrancy(window) {
                eprintln!("[mochi] app window vibrancy unavailable: {error}");
            }
        })
        .map_err(|error| error.to_string())
}

fn ensure_main_panel_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(MAIN_PANEL_LABEL) {
        return Ok(window);
    }

    let builder = WebviewWindowBuilder::new(app, MAIN_PANEL_LABEL, WebviewUrl::App("/".into()))
        .title("Mochi")
        .inner_size(TRAY_PANEL_WIDTH, TRAY_PANEL_MIN_HEIGHT)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(window_uses_native_transparency())
        .shadow(false)
        .visible(false);

    builder.build().map_err(|error| error.to_string())
}

pub fn setup_main_panel(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(TrayIconRectState::default());

    let window = ensure_main_panel_window(app)?;
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

    if let Err(error) = ensure_tray_panel_vibrancy(window) {
        eprintln!("[mochi] tray panel vibrancy unavailable: {error}");
    }

    let _ = window.hide();
    Ok(())
}

fn ensure_tray_panel_vibrancy(window: &WebviewWindow) -> Result<(), String> {
    super::vibrancy::apply_tray_panel_vibrancy(window)
}

fn ensure_app_window_vibrancy(window: &WebviewWindow) -> Result<(), String> {
    super::vibrancy::apply_app_window_vibrancy(window)
}

fn app_window_size_for_path(path: &str) -> (f64, f64) {
    if path.starts_with("/about") {
        (ABOUT_WINDOW_WIDTH, ABOUT_WINDOW_HEIGHT)
    } else if path.starts_with("/update") {
        (UPDATE_WINDOW_WIDTH, UPDATE_WINDOW_HEIGHT)
    } else {
        (SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
    }
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
    app.try_state::<TrayIconRectState>()
        .is_some_and(|state| state.0.lock().ok().is_some_and(|cached| cached.is_some()))
}

/// Resizes the tray popover to content height (logical px), reclamping anchor when visible.
#[tauri::command]
pub fn set_tray_panel_height(app: AppHandle, height: f64) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_PANEL_LABEL) else {
        return Err(format!("missing tray panel window: {MAIN_PANEL_LABEL}"));
    };

    let max_height = tray_panel_max_height(&window);
    let height = height.clamp(TRAY_PANEL_MIN_HEIGHT, max_height);
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

fn tray_panel_max_height(window: &WebviewWindow) -> f64 {
    window
        .current_monitor()
        .ok()
        .flatten()
        .map(|monitor| {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let logical_height = size.height as f64 / scale;
            (logical_height - TRAY_PANEL_VIEWPORT_MARGIN).max(TRAY_PANEL_MIN_HEIGHT)
        })
        .unwrap_or(TRAY_PANEL_DEFAULT_MAX_HEIGHT)
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
        let _ = emit_tray_navigate(&app, "/");
        let _ = tray_panel.hide();
    }

    let window = ensure_settings_window(&app)?;

    let (width, height) = app_window_size_for_path(path.as_str());
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
    let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
        width: if path.starts_with("/about") {
            ABOUT_WINDOW_WIDTH
        } else if path.starts_with("/update") {
            UPDATE_WINDOW_WIDTH
        } else {
            480.0
        },
        height: if path.starts_with("/about") {
            ABOUT_WINDOW_HEIGHT
        } else if path.starts_with("/update") {
            UPDATE_WINDOW_HEIGHT
        } else {
            420.0
        },
    })));

    if let Err(error) = ensure_app_window_vibrancy(&window) {
        eprintln!("[mochi] app window vibrancy unavailable: {error}");
    }

    emit_app_navigate(&app, path.as_str())?;

    #[cfg(target_os = "macos")]
    set_regular_activation_policy(&app);

    let _ = window.set_skip_taskbar(false);
    crate::app_branding::sync_app_window_branding(&app, &window);

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

    let _ = emit_tray_navigate(app, path);

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

    let _ = emit_tray_navigate(app, path);

    if window.is_visible().unwrap_or(false) {
        let _ = window.set_focus();
        return;
    }

    open_visible_tray_panel(app, &window);
}

fn open_visible_tray_panel(app: &AppHandle, window: &WebviewWindow) {
    if let Err(error) = ensure_tray_panel_vibrancy(window) {
        eprintln!("[mochi] tray panel vibrancy unavailable: {error}");
    }

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

    let _ = emit_tray_navigate(app, path);
    if let Err(error) = ensure_tray_panel_vibrancy(&window) {
        eprintln!("[mochi] tray panel vibrancy unavailable: {error}");
    }
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
    use std::sync::Mutex;

    use super::*;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn dev_show_main_env_var_behavior() {
        let _guard = ENV_TEST_LOCK.lock().expect("env test lock");

        std::env::remove_var("MOCHI_DEV_SHOW_MAIN");
        assert!(!dev_show_main_enabled());

        std::env::set_var("MOCHI_DEV_SHOW_MAIN", "1");
        assert!(dev_show_main_enabled());

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
        assert!(matches!(tray_panel_fallback_position(), Position::TopRight));

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

    #[test]
    fn app_window_size_for_path() {
        assert_eq!(
            super::app_window_size_for_path("/settings"),
            (SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
        );
        assert_eq!(
            super::app_window_size_for_path("/about"),
            (ABOUT_WINDOW_WIDTH, ABOUT_WINDOW_HEIGHT)
        );
        assert_eq!(
            super::app_window_size_for_path("/update"),
            (UPDATE_WINDOW_WIDTH, UPDATE_WINDOW_HEIGHT)
        );
    }

    #[test]
    fn navigation_events_target_dedicated_window_labels() {
        assert_eq!(MAIN_PANEL_LABEL, "main");
        assert_eq!(SETTINGS_WINDOW_LABEL, "settings");
    }
}
