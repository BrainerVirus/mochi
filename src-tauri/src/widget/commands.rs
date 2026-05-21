use tauri::{AppHandle, Manager, WebviewUrl};

use super::{WIDGET_LABEL, WIDGET_MAX_WIDTH, WIDGET_MIN_HEIGHT, WIDGET_MIN_WIDTH};

pub fn setup_widget(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if app.get_webview_window(WIDGET_LABEL).is_some() {
        prepare_widget_window(app)?;
        return Ok(());
    }

    let url = WebviewUrl::App("/widget".into());
    let window = tauri::WebviewWindowBuilder::new(app, WIDGET_LABEL, url)
        .title("Mochi Widget")
        .inner_size(320.0, 420.0)
        .min_inner_size(WIDGET_MIN_WIDTH, WIDGET_MIN_HEIGHT)
        .max_inner_size(WIDGET_MAX_WIDTH, 720.0)
        .resizable(true)
        .visible(false)
        .always_on_top(true)
        .build()?;

    let _ = window;
    Ok(())
}

fn prepare_widget_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        let _ = window.set_always_on_top(true);
        let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: WIDGET_MIN_WIDTH,
            height: WIDGET_MIN_HEIGHT,
        })));
        let _ = window.set_max_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: WIDGET_MAX_WIDTH,
            height: 720.0,
        })));
        let _ = window.hide();
    }

    Ok(())
}

#[tauri::command]
pub fn show_widget(app: AppHandle) -> Result<(), String> {
    widget_window(&app)?
        .show()
        .map_err(|error| error.to_string())?;
    widget_window(&app)?
        .set_focus()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn hide_widget(app: AppHandle) -> Result<(), String> {
    widget_window(&app)?
        .hide()
        .map_err(|error| error.to_string())
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
