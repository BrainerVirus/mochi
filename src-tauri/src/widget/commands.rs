use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::diagnostics::DiagnosticsState;
use crate::frontend::app_shell_url;
use crate::settings::SettingsState;

use super::{WIDGET_LABEL, WIDGET_MIN_HEIGHT, WIDGET_MIN_WIDTH, WIDGET_WIDTH};

fn logical_outer_size(window: &WebviewWindow) -> Option<(f64, f64)> {
    let scale = window.scale_factor().ok()?;
    let size = window.outer_size().ok()?;
    Some((
        f64::from(size.width) / scale,
        f64::from(size.height) / scale,
    ))
}

fn logical_inner_size(window: &WebviewWindow) -> Option<(f64, f64)> {
    let scale = window.scale_factor().ok()?;
    let size = window.inner_size().ok()?;
    Some((
        f64::from(size.width) / scale,
        f64::from(size.height) / scale,
    ))
}

fn record_widget_window_lifecycle(
    window: &WebviewWindow,
    phase: &str,
    creation: &str,
    initial_visibility: &str,
) {
    if let Some(state) = window
        .app_handle()
        .try_state::<crate::diagnostics::DiagnosticsState>()
    {
        let policy = crate::window_policy::active_decorated_window_policy().name;
        state.record_window_lifecycle(
            WIDGET_LABEL,
            phase,
            policy,
            creation,
            initial_visibility,
            logical_outer_size(window),
            logical_inner_size(window),
        );
    }
}

pub fn setup_widget(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if app.get_webview_window(WIDGET_LABEL).is_some() {
        prepare_widget_window(app)?;
        return Ok(());
    }

    let window = build_widget_window(app)?;
    let policy = crate::window_policy::active_decorated_window_policy();
    record_widget_window_lifecycle(
        &window,
        "created",
        policy.creation_label(),
        policy.initial_visibility_label(),
    );
    record_widget_window_controls(app, &window, "rust-builder");
    if let Some(state) = app.try_state::<DiagnosticsState>() {
        let url = window
            .url()
            .map(|parsed| parsed.to_string())
            .unwrap_or_else(|_| "unknown".into());
        state.record_window_created(WIDGET_LABEL, &url, true, false);
    }
    Ok(())
}

fn build_widget_window(app: &AppHandle) -> Result<WebviewWindow, tauri::Error> {
    let selected_tab = app
        .try_state::<SettingsState>()
        .and_then(|state| state.current().ok())
        .and_then(|s| s.selected_tab.clone())
        .unwrap_or_default();

    let init_script = if selected_tab.is_empty() {
        String::new()
    } else {
        format!(
            "window.__MOCHI_SELECTED_TAB__ = '{}';",
            selected_tab.replace('\'', "\\'")
        )
    };

    let mut builder = tauri::WebviewWindowBuilder::new(app, WIDGET_LABEL, app_shell_url())
        .title("Mochi Widget")
        .inner_size(WIDGET_WIDTH, 420.0)
        .min_inner_size(WIDGET_MIN_WIDTH, WIDGET_MIN_HEIGHT)
        .decorations(true)
        .resizable(true)
        .visible(matches!(
            crate::window_policy::decorated_window_initial_visibility(),
            crate::window_policy::DecoratedWindowInitialVisibility::Visible
        ));

    if !init_script.is_empty() {
        builder = builder.initialization_script(&init_script);
    }

    builder.build()
}

fn prepare_widget_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        record_widget_window_controls(app, &window, "rust-builder");
        let _ = window.set_always_on_top(false);
        let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: WIDGET_MIN_WIDTH,
            height: WIDGET_MIN_HEIGHT,
        })));
    }

    Ok(())
}

#[tauri::command]
pub fn show_widget(app: AppHandle) -> Result<(), String> {
    let already_existed = app.get_webview_window(WIDGET_LABEL).is_some();
    let window = ensure_widget_window(&app)?;

    // Emit current selected tab before showing (only for reused windows).
    // First creation handles this via initialization_script which runs before any
    // page JS. The emit_to would be lost on first creation since the webview hasn't
    // loaded the page — the already_existed guard prevents a wasted event.
    if already_existed {
        if let Some(state) = app.try_state::<SettingsState>() {
            if let Ok(settings) = state.current() {
                if let Some(tab) = &settings.selected_tab {
                    let _ = app.emit_to(WIDGET_LABEL, "set-tab", tab);
                }
            }
        }
    }

    let policy = crate::window_policy::active_decorated_window_policy();
    let creation = policy.creation_label();
    let initial_visibility = policy.initial_visibility_label();
    record_widget_window_controls(&app, &window, creation);

    match crate::window_policy::first_show_sequence() {
        crate::window_policy::FirstShowSequence::AlreadyVisibleFocus => {
            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "set_focus",
                focus_result.as_ref().map(|_| ()),
            );
            record_widget_window_lifecycle(&window, "after-focus", creation, initial_visibility);
            focus_result.map_err(|error| error.to_string())
        }
        crate::window_policy::FirstShowSequence::ShowUnminimizeFocus => {
            let show_result = window.show();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "show",
                show_result.as_ref().map(|_| ()),
            );
            show_result.map_err(|error| error.to_string())?;
            record_widget_window_lifecycle(&window, "after-show", creation, initial_visibility);

            let unminimize_result = window.unminimize();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "unminimize",
                unminimize_result.as_ref().map(|_| ()),
            );
            unminimize_result.map_err(|error| error.to_string())?;
            record_widget_window_lifecycle(
                &window,
                "after-unminimize",
                creation,
                initial_visibility,
            );
            record_widget_window_controls(&app, &window, creation);

            let focus_result = window.set_focus();
            crate::diagnostics::log_window_action_result(
                WIDGET_LABEL,
                "set_focus",
                focus_result.as_ref().map(|_| ()),
            );
            record_widget_window_lifecycle(&window, "after-focus", creation, initial_visibility);
            focus_result.map_err(|error| error.to_string())
        }
    }
}

#[tauri::command]
pub fn set_widget_height(app: AppHandle, height: f64) -> Result<(), String> {
    let window = widget_window(&app)?;
    let width = widget_logical_width(&window);
    let height = height.clamp(WIDGET_MIN_HEIGHT, 720.0);
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn hide_widget(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(WIDGET_LABEL) else {
        return Ok(());
    };
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
    let window = ensure_widget_window(&app)?;
    if window.is_visible().map_err(|error| error.to_string())? {
        hide_widget(app)
    } else {
        show_widget(app)
    }
}

fn widget_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(WIDGET_LABEL)
        .ok_or_else(|| format!("missing widget window: {WIDGET_LABEL}"))
}

fn ensure_widget_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        return Ok(window);
    }

    let window = build_widget_window(app).map_err(|error| error.to_string())?;
    let policy = crate::window_policy::active_decorated_window_policy();
    record_widget_window_lifecycle(
        &window,
        "created",
        policy.creation_label(),
        policy.initial_visibility_label(),
    );
    record_widget_window_controls(app, &window, "rust-builder");
    if let Some(state) = app.try_state::<DiagnosticsState>() {
        let url = window
            .url()
            .map(|parsed| parsed.to_string())
            .unwrap_or_else(|_| "unknown".into());
        state.record_window_created(WIDGET_LABEL, &url, true, false);
    }
    Ok(window)
}

fn record_widget_window_controls(app: &AppHandle, window: &WebviewWindow, creation_source: &str) {
    let diagnostics = crate::linux_window_controls::prepare_decorated_window(
        window,
        WIDGET_LABEL,
        creation_source,
    );
    if let Some(state) = app.try_state::<DiagnosticsState>() {
        state.record_linux_window_controls(diagnostics);
    }
}

fn widget_logical_width(window: &WebviewWindow) -> f64 {
    let scale_factor = window.scale_factor().unwrap_or(1.0);
    let width = window
        .inner_size()
        .map(|size| f64::from(size.width) / scale_factor)
        .unwrap_or(WIDGET_WIDTH);
    width.max(WIDGET_MIN_WIDTH)
}
