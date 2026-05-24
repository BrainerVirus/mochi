mod icon;
#[cfg(target_os = "macos")]
mod macos_window_shape;
mod menu_bar_metric;
mod panel;
mod presentation;
mod usage;
mod vibrancy;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

use crate::core::models::UsageSnapshot;
use crate::core::usage_store::UsageStore;
use crate::settings::SettingsState;
use crate::status::read_cached_snapshots;

pub use panel::{
    maybe_show_main_for_dev, open_app_window, open_tray_panel, record_tray_icon_event,
    set_tray_panel_height, setup_app_windows, setup_main_panel, show_main_panel, show_tray_panel,
    show_tray_panel_centered, MAIN_PANEL_LABEL, SETTINGS_WINDOW_LABEL,
};
pub use presentation::{
    pick_tray_snapshot, resolve_tray_presentation, TrayIconPresentation, TraySelection,
};
pub use usage::{aggregate_used_percent, tray_usage_tone, TrayUsageTone, TRAY_ID};

use icon::tray_icon_for_presentation;

pub fn apply_tray_usage(
    app: &AppHandle,
    snapshots: &[UsageSnapshot],
    selection: TraySelection,
) -> Result<(), String> {
    let presentation = resolve_tray_presentation(snapshots, selection);
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| format!("tray icon {TRAY_ID} not found"))?;

    let icon = tray_icon_for_presentation(&presentation);

    tray.set_tooltip(Some(presentation.tooltip.clone()))
        .map_err(|error| error.to_string())?;
    tray.set_icon(Some(icon))
        .map_err(|error| error.to_string())?;
    // Crisp system font beside template icon (macOS); no percent baked into RGBA.
    tray.set_title(presentation.title.as_deref())
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn sync_tray_usage(
    app: AppHandle,
    settings_state: State<'_, SettingsState>,
    usage_store: State<'_, UsageStore>,
    selection: Option<String>,
) -> Result<(), String> {
    let settings = settings_state.current()?;
    let snapshots = read_cached_snapshots(&usage_store, &settings);

    let tray_selection = TraySelection::parse(selection.as_deref());
    apply_tray_usage(&app, &snapshots, tray_selection)
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh usage", true, None::<&str>)?;
    let widget_item = MenuItem::with_id(app, "widget", "Show widget", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let stable_channel_item =
        MenuItem::with_id(app, "channel-stable", "Stable", true, None::<&str>)?;
    let unstable_channel_item =
        MenuItem::with_id(app, "channel-unstable", "Unstable", true, None::<&str>)?;
    let channel_menu = Submenu::with_items(
        app,
        "Update channel",
        true,
        &[&stable_channel_item, &unstable_channel_item],
    )?;
    let update_item = MenuItem::with_id(app, "update", "Check for updates", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Mochi", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &refresh_item,
            &widget_item,
            &settings_item,
            &channel_menu,
            &update_item,
            &separator,
            &quit_item,
        ],
    )?;

    let icon = icon::tray_icon_fallback();

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Mochi")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "refresh" => {
                let _ = app.emit("tray-refresh", ());
            }
            "widget" => {
                let _ = crate::widget::show_widget(app.clone());
            }
            "settings" => {
                let _ = open_app_window(app.clone(), "/settings".to_string());
            }
            "channel-stable" => {
                let _ = app.emit("tray-set-channel", "stable");
            }
            "channel-unstable" => {
                let _ = app.emit("tray-set-channel", "unstable");
            }
            "update" => {
                let _ = app.emit("tray-check-update", ());
            }
            "quit" => {
                if let Some(lifecycle) = app.try_state::<crate::lifecycle::AppLifecycle>() {
                    lifecycle.request_quit();
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            record_tray_icon_event(tray.app_handle(), &event);

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_tray_panel(tray.app_handle(), "/");
            }
        })
        .build(app)?;

    #[cfg(debug_assertions)]
    eprintln!("[mochi] tray registered (id={TRAY_ID})");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};

    fn snapshot(used_percent: f32) -> UsageSnapshot {
        UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", used_percent, None),
            None,
            "2026-05-20T12:00:00Z",
            "test",
        )
    }

    #[test]
    fn resolve_tray_presentation_uses_remaining_percent_in_tooltip() {
        let snapshots = vec![snapshot(12.0), snapshot(88.0)];
        let presentation = resolve_tray_presentation(&snapshots, TraySelection::Overview);
        assert_eq!(presentation.remaining_percent, 50);
        assert!(presentation.tooltip.contains("50% left"));
    }
}
