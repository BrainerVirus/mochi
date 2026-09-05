use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

fn should_send_notification(show_notifications: bool) -> bool {
    show_notifications
}

pub fn send_notification(app: &AppHandle, title: &str, body: &str) {
    let enabled = app
        .try_state::<crate::settings::SettingsState>()
        .and_then(|state| state.current().ok())
        .map(|settings| settings.show_notifications)
        .unwrap_or(true);
    if !should_send_notification(enabled) {
        return;
    }
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        crate::diagnostics::log_line("notify", &format!("send failed: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_master_toggle_suppresses_notification() {
        assert!(!should_send_notification(false));
    }

    #[test]
    fn enabled_master_toggle_allows_notification() {
        assert!(should_send_notification(true));
    }
}
