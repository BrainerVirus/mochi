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
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

use crate::core::models::UsageSnapshot;
use crate::core::usage_store::UsageStore;
use crate::settings::{MochiSettings, SettingsState};
use crate::status::{read_cached_snapshots, RefreshCompletePayload};

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
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrayMenuModel {
    entries: Vec<TrayMenuEntry>,
}

fn build_tray_menu_model() -> TrayMenuModel {
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

fn refresh_tray_selection(settings: &MochiSettings) -> TraySelection {
    TraySelection::parse(settings.selected_tab.as_deref())
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

fn build_menu_from_model(
    app: &AppHandle,
    model: &TrayMenuModel,
) -> Result<Menu<Runtime>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;

    for entry in &model.entries {
        match entry {
            TrayMenuEntry::Item { id, label } => {
                let item = MenuItem::with_id(app, *id, *label, true, None::<&str>)?;
                menu.append(&item)?;
            }
            TrayMenuEntry::Separator => {
                let separator = PredefinedMenuItem::separator(app)?;
                menu.append(&separator)?;
            }
        }
    }

    Ok(menu)
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let model = build_tray_menu_model();
    let menu = build_menu_from_model(app, &model)?;

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
                                let payload =
                                    crate::status::refresh_all_providers_inner(&store, &settings)
                                        .await
                                        .unwrap_or_else(|_| RefreshCompletePayload {
                                            states: crate::status::read_cached_usage_states(
                                                &store, &settings,
                                            ),
                                        });
                                let _ = app.emit("usage-refresh-complete", &payload);
                                let snapshots = read_cached_snapshots(&store, &settings);
                                let selection = refresh_tray_selection(&settings);
                                let _ = apply_tray_usage(&app, &snapshots, selection);
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
    use crate::settings::MochiSettings;

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
    fn refresh_tray_selection_preserves_the_saved_provider() {
        let settings = MochiSettings {
            selected_tab: Some("codex".into()),
            ..MochiSettings::default()
        };

        assert_eq!(
            refresh_tray_selection(&settings),
            TraySelection::Provider(ProviderId::Codex)
        );
    }

    #[test]
    fn tray_menu_model_removes_show_usage_and_prioritizes_widget() {
        let model = build_tray_menu_model();
        let labels = tray_menu_labels(&model);
        assert_eq!(labels.first(), Some(&"Open widget"));
        assert!(!labels.contains(&"Show usage"));
        assert!(!labels.contains(&"Show widget"));
        assert!(!labels.contains(&"Update channel"));
        assert!(labels.contains(&"Refresh usage"));
        assert!(labels.contains(&"Settings"));
    }

    #[test]
    fn tray_menu_model_has_no_channel_items() {
        let ids: Vec<&str> = build_tray_menu_model()
            .entries
            .iter()
            .filter_map(|entry| match entry {
                TrayMenuEntry::Item { id, .. } => Some(*id),
                TrayMenuEntry::Separator => None,
            })
            .collect();
        assert_eq!(ids, vec!["widget", "refresh", "settings", "update", "quit"]);
    }

    fn tray_menu_labels(model: &TrayMenuModel) -> Vec<&'static str> {
        fn collect(entry: &TrayMenuEntry, labels: &mut Vec<&'static str>) {
            match entry {
                TrayMenuEntry::Item { label, .. } => labels.push(label),
                TrayMenuEntry::Separator => {}
            }
        }

        let mut labels = Vec::new();
        for entry in &model.entries {
            collect(entry, &mut labels);
        }
        labels
    }
}
