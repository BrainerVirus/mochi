mod icon;
#[cfg(target_os = "macos")]
mod macos_window_shape;
mod menu_bar_metric;
mod panel;
mod presentation;
mod usage;
mod vibrancy;
mod window_transparency;

use tauri::{
    menu::{CheckMenuItem, CheckMenuItemBuilder, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

use crate::core::models::UsageSnapshot;
use crate::core::usage_store::UsageStore;
use crate::settings::{SettingsState, UpdateChannel};
use crate::status::read_cached_snapshots;

pub use panel::{
    maybe_show_main_for_dev, open_app_window, open_tray_panel, record_tray_icon_event,
    set_tray_panel_height, setup_app_windows, setup_main_panel, show_main_panel, show_tray_panel,
    show_tray_panel_centered, MAIN_PANEL_LABEL, SETTINGS_WINDOW_LABEL,
};
pub use presentation::{
    pick_tray_snapshot, provider_display_name, resolve_tray_presentation, TrayIconPresentation,
    TraySelection,
};
pub use usage::{aggregate_used_percent, tray_usage_tone, TrayUsageTone, TRAY_ID};

use icon::tray_icon_for_presentation;

type Runtime = tauri::Wry;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TrayMenuEntry {
    Item {
        id: &'static str,
        label: &'static str,
    },
    Channel {
        id: &'static str,
        label: &'static str,
        checked: bool,
    },
    Submenu {
        label: &'static str,
        children: Vec<TrayMenuEntry>,
    },
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrayMenuModel {
    entries: Vec<TrayMenuEntry>,
}

#[derive(Clone)]
pub struct TrayChannelMenuState {
    stable: CheckMenuItem<Runtime>,
    unstable: CheckMenuItem<Runtime>,
}

impl TrayChannelMenuState {
    fn set_channel(&self, channel: &str) -> Result<(), String> {
        let unstable = channel == "unstable";
        self.stable
            .set_checked(!unstable)
            .map_err(|error| error.to_string())?;
        self.unstable
            .set_checked(unstable)
            .map_err(|error| error.to_string())
    }
}

fn build_tray_menu_model(channel: &str) -> TrayMenuModel {
    let unstable = channel == "unstable";
    TrayMenuModel {
        entries: vec![
            TrayMenuEntry::Item {
                id: "widget",
                label: "Open widget",
            },
            TrayMenuEntry::Item {
                id: "refresh",
                label: "Refresh usage",
            },
            TrayMenuEntry::Item {
                id: "settings",
                label: "Settings",
            },
            TrayMenuEntry::Submenu {
                label: "Update channel",
                children: vec![
                    TrayMenuEntry::Channel {
                        id: "channel-stable",
                        label: "Stable",
                        checked: !unstable,
                    },
                    TrayMenuEntry::Channel {
                        id: "channel-unstable",
                        label: "Unstable",
                        checked: unstable,
                    },
                ],
            },
            TrayMenuEntry::Item {
                id: "update",
                label: "Check for updates",
            },
            TrayMenuEntry::Separator,
            TrayMenuEntry::Item {
                id: "quit",
                label: "Quit Mochi",
            },
        ],
    }
}

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

#[tauri::command]
pub fn sync_tray_update_channel(
    channel: String,
    state: State<'_, TrayChannelMenuState>,
) -> Result<(), String> {
    state.set_channel(channel.as_str())
}

fn build_menu_from_model(
    app: &AppHandle,
    model: &TrayMenuModel,
) -> Result<(Menu<Runtime>, TrayChannelMenuState), Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;
    let mut stable_channel_item = None;
    let mut unstable_channel_item = None;

    for entry in &model.entries {
        match entry {
            TrayMenuEntry::Item { id, label } => {
                let item = MenuItem::with_id(app, *id, *label, true, None::<&str>)?;
                menu.append(&item)?;
            }
            TrayMenuEntry::Channel { .. } => {}
            TrayMenuEntry::Submenu { label, children } => {
                let submenu = Submenu::new(app, *label, true)?;
                for child in children {
                    if let TrayMenuEntry::Channel { id, label, checked } = child {
                        let item = CheckMenuItemBuilder::with_id(*id, *label)
                            .checked(*checked)
                            .build(app)?;
                        if *id == "channel-stable" {
                            stable_channel_item = Some(item.clone());
                        } else if *id == "channel-unstable" {
                            unstable_channel_item = Some(item.clone());
                        }
                        submenu.append(&item)?;
                    }
                }
                menu.append(&submenu)?;
            }
            TrayMenuEntry::Separator => {
                let separator = PredefinedMenuItem::separator(app)?;
                menu.append(&separator)?;
            }
        }
    }

    let state = TrayChannelMenuState {
        stable: stable_channel_item.ok_or("missing stable channel menu item")?,
        unstable: unstable_channel_item.ok_or("missing unstable channel menu item")?,
    };

    Ok((menu, state))
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let current_channel = app
        .try_state::<SettingsState>()
        .and_then(|state| state.current().ok())
        .map(|settings| match settings.update_channel {
            UpdateChannel::Stable => "stable".to_string(),
            UpdateChannel::Unstable => "unstable".to_string(),
        })
        .unwrap_or_else(|| "stable".to_string());
    let model = build_tray_menu_model(&current_channel);
    let (menu, channel_state) = build_menu_from_model(app, &model)?;
    app.manage(channel_state);

    let icon = icon::tray_icon_fallback();

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Mochi")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "refresh" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(store) = app.try_state::<crate::core::usage_store::UsageStore>() {
                        if let Some(settings_state) =
                            app.try_state::<crate::settings::SettingsState>()
                        {
                            if let Ok(settings) = settings_state.current() {
                                let _ =
                                    crate::status::refresh_all_providers_inner(&store, &settings)
                                        .await
                                        .map(|payload| {
                                            let _ = app.emit("usage-refresh-complete", &payload);
                                        });
                            }
                        }
                    }
                });
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

    #[test]
    fn tray_menu_model_removes_show_usage_and_prioritizes_widget() {
        let model = build_tray_menu_model("stable");
        let labels = tray_menu_labels(&model);
        assert_eq!(labels.first(), Some(&"Open widget"));
        assert!(!labels.contains(&"Show usage"));
        assert!(!labels.contains(&"Show widget"));
        assert!(labels.contains(&"Refresh usage"));
        assert!(labels.contains(&"Settings"));
        assert!(labels.contains(&"Update channel"));
    }

    #[test]
    fn tray_menu_model_marks_current_channel() {
        assert_eq!(
            checked_channel_id(&build_tray_menu_model("stable")),
            Some("channel-stable")
        );
        assert_eq!(
            checked_channel_id(&build_tray_menu_model("unstable")),
            Some("channel-unstable")
        );
        assert_eq!(
            checked_channel_id(&build_tray_menu_model("unexpected")),
            Some("channel-stable")
        );
    }

    fn tray_menu_labels(model: &TrayMenuModel) -> Vec<&'static str> {
        fn collect(entry: &TrayMenuEntry, labels: &mut Vec<&'static str>) {
            match entry {
                TrayMenuEntry::Item { label, .. }
                | TrayMenuEntry::Channel { label, .. }
                | TrayMenuEntry::Submenu { label, .. } => labels.push(label),
                TrayMenuEntry::Separator => {}
            }

            if let TrayMenuEntry::Submenu { children, .. } = entry {
                for child in children {
                    collect(child, labels);
                }
            }
        }

        let mut labels = Vec::new();
        for entry in &model.entries {
            collect(entry, &mut labels);
        }
        labels
    }

    fn checked_channel_id(model: &TrayMenuModel) -> Option<&'static str> {
        model.entries.iter().find_map(|entry| match entry {
            TrayMenuEntry::Submenu { children, .. } => {
                children.iter().find_map(|child| match child {
                    TrayMenuEntry::Channel {
                        id, checked: true, ..
                    } => Some(*id),
                    _ => None,
                })
            }
            _ => None,
        })
    }
}
