use tauri::{AppHandle, Manager, WebviewWindow, WindowEvent};

use crate::diagnostics::{log_line, DiagnosticsState};
use crate::frontend::app_shell_url;

use super::{WIDGET_LABEL, WIDGET_MAX_WIDTH, WIDGET_MIN_HEIGHT, WIDGET_MIN_WIDTH};

pub fn setup_widget(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if app.get_webview_window(WIDGET_LABEL).is_some() {
        prepare_widget_window(app)?;
        return Ok(());
    }

    let window = tauri::WebviewWindowBuilder::new(app, WIDGET_LABEL, app_shell_url())
        .title("Mochi Widget")
        .inner_size(320.0, 420.0)
        .min_inner_size(WIDGET_MIN_WIDTH, WIDGET_MIN_HEIGHT)
        .max_inner_size(WIDGET_MAX_WIDTH, 720.0)
        .resizable(true)
        .visible(false)
        .always_on_top(true)
        .build()?;

    prepare_widget_window_events(&window)?;
    crate::linux_window_controls::prepare_decorated_window(&window, WIDGET_LABEL);
    if let Some(state) = app.try_state::<DiagnosticsState>() {
        let url = window
            .url()
            .map(|parsed| parsed.to_string())
            .unwrap_or_else(|_| "unknown".into());
        state.record_window_created(WIDGET_LABEL, &url, true, false);
    }
    Ok(())
}

fn prepare_widget_window_events(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    let window_for_events = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let app = window_for_events.app_handle();
            if let Some(state) = app.try_state::<DiagnosticsState>() {
                state.record_window_event(WIDGET_LABEL, "close_requested -> hide");
            }
            log_line(
                "window",
                &format!("{WIDGET_LABEL}: close_requested -> hide"),
            );
            let hide_result = window_for_events.hide();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "hide",
                hide_result.as_ref().map(|_| ()),
            );
        }
    });
    Ok(())
}

fn prepare_widget_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        prepare_widget_window_events(&window)?;
        crate::linux_window_controls::prepare_decorated_window(&window, WIDGET_LABEL);
        let _ = window.set_always_on_top(true);
        let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: WIDGET_MIN_WIDTH,
            height: WIDGET_MIN_HEIGHT,
        })));
        let _ = window.set_max_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: WIDGET_MAX_WIDTH,
            height: 720.0,
        })));
        let hide_result = window.hide();
        crate::diagnostics::log_window_action_result(
            WIDGET_LABEL,
            "hide_prepare",
            hide_result.as_ref().map(|_| ()),
        );
    }

    Ok(())
}

#[tauri::command]
pub fn show_widget(app: AppHandle) -> Result<(), String> {
    let window = widget_window(&app)?;
    crate::linux_window_controls::prepare_decorated_window(&window, WIDGET_LABEL);

    let show_result = window.show();
    crate::diagnostics::log_window_action_result(
        WIDGET_LABEL,
        "show",
        show_result.as_ref().map(|_| ()),
    );
    show_result.map_err(|error| error.to_string())?;

    let unminimize_result = window.unminimize();
    crate::diagnostics::log_window_action_result(
        WIDGET_LABEL,
        "unminimize",
        unminimize_result.as_ref().map(|_| ()),
    );
    unminimize_result.map_err(|error| error.to_string())?;
    crate::linux_window_controls::prepare_decorated_window(&window, WIDGET_LABEL);

    let focus_result = window.set_focus();
    crate::diagnostics::log_window_action_result(
        WIDGET_LABEL,
        "set_focus",
        focus_result.as_ref().map(|_| ()),
    );
    focus_result.map_err(|error| error.to_string())
}

#[tauri::command]
pub fn hide_widget(app: AppHandle) -> Result<(), String> {
    let window = widget_window(&app)?;
    let hide_result = window.hide();
    crate::diagnostics::log_window_action_result(
        WIDGET_LABEL,
        "hide",
        hide_result.as_ref().map(|_| ()),
    );
    hide_result.map_err(|error| error.to_string())
}

#[tauri::command]
pub fn toggle_widget(app: AppHandle) -> Result<(), String> {
    let window = widget_window(&app)?;
    if window.is_visible().map_err(|error| error.to_string())? {
        hide_widget(app)
    } else {
        show_widget(app)
    }
}

fn widget_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window(WIDGET_LABEL)
        .ok_or_else(|| format!("missing widget window: {WIDGET_LABEL}"))
}
