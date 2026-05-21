use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn tray_tooltip(used_percent: u8) -> String {
    format!("Mochi: {used_percent}% used")
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh usage", true, None::<&str>)?;
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
            &settings_item,
            &channel_menu,
            &update_item,
            &separator,
            &quit_item,
        ],
    )?;

    let icon = app
        .default_window_icon()
        .ok_or("missing default window icon")?
        .clone();

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip(tray_tooltip(0))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "refresh" => {
                let _ = app.emit("tray-refresh", ());
            }
            "settings" => {
                show_main_window(app, "/settings");
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
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle(), "/");
            }
        })
        .build(app)?;

    Ok(())
}

fn show_main_window(app: &AppHandle, path: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("tray-navigate", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_tooltip_includes_used_percent() {
        assert_eq!(tray_tooltip(42), "Mochi: 42% used");
    }
}
