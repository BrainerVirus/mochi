mod icon;
mod panel;
mod usage;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, State,
};

use crate::core::models::UsageSnapshot;
use crate::settings::SettingsState;
use crate::status;

pub use panel::{
    maybe_show_main_for_dev, open_tray_panel, record_tray_icon_event, setup_main_panel,
    show_main_panel, show_tray_panel, show_tray_panel_centered, MAIN_PANEL_LABEL,
};
pub use usage::{aggregate_used_percent, tray_usage_tone, TrayUsageTone, TRAY_ID};

use icon::tray_icon_for_tone;

pub fn tray_tooltip(used_percent: u8) -> String {
    format!("Mochi: {used_percent}% used")
}

pub fn apply_tray_usage(app: &AppHandle, snapshots: &[UsageSnapshot]) -> Result<(), String> {
    let used_percent = aggregate_used_percent(snapshots);
    let tone = tray_usage_tone(used_percent);
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| format!("tray icon {TRAY_ID} not found"))?;

    tray.set_tooltip(Some(tray_tooltip(used_percent)))
        .map_err(|error| error.to_string())?;
    tray.set_icon(Some(tray_icon_for_tone(tone)))
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn sync_tray_usage(
    app: AppHandle,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    let settings = state.current()?;
    let snapshots = status::collect_usage_snapshots(&settings.enabled_providers)
        .await
        .map_err(|error| error.to_string())?;

    apply_tray_usage(&app, &snapshots)
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

    let icon = tray_icon_for_tone(TrayUsageTone::Normal);

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(tray_tooltip(0))
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
                open_tray_panel(app, "/settings");
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
        UsageSnapshot {
            provider: ProviderId::Claude,
            primary: UsageWindow::new("Session", used_percent, None),
            secondary: None,
            updated_at: "1970-01-01T00:00:00Z".to_string(),
            source: "test".to_string(),
        }
    }

    #[test]
    fn tray_tooltip_includes_used_percent() {
        assert_eq!(tray_tooltip(42), "Mochi: 42% used");
    }

    #[test]
    fn apply_tray_usage_aggregates_snapshots_for_tooltip_text() {
        let snapshots = vec![snapshot(12.0), snapshot(88.0)];
        assert_eq!(aggregate_used_percent(&snapshots), 88);
        assert_eq!(tray_usage_tone(88), TrayUsageTone::Critical);
        assert_eq!(
            tray_tooltip(aggregate_used_percent(&snapshots)),
            "Mochi: 88% used"
        );
    }
}
