use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::settings::{self, MochiSettings};

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

/// In-memory one-shot state per threshold key (provider id or "overall");
/// missing entries start armed.
fn threshold_armed() -> &'static Mutex<HashMap<String, bool>> {
    static ARMED: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    ARMED.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Pure threshold evaluation over the same snapshot percentages the widget
/// renders. Returns (title, body) pairs and updates `armed`: a crossing fires
/// once and disarms until usage dips back below the threshold. The overall
/// entry reuses the Overview aggregate (`aggregate_used_percent`), no new metric.
pub fn evaluate_threshold_notifications(
    settings: &MochiSettings,
    snapshots: &[UsageSnapshot],
    armed: &mut HashMap<String, bool>,
) -> Vec<(String, String)> {
    let mut notifications = Vec::new();

    for id in &settings.enabled_providers {
        let Some(provider) = ProviderId::parse(id) else {
            continue;
        };
        let threshold = settings.effective_warn_percent(provider);
        let Some(snapshot) = snapshots.iter().find(|s| s.provider == provider) else {
            continue;
        };
        let usage = snapshot.primary.used_percent as f64;
        let key = provider.as_str().to_string();
        let is_armed = armed.get(&key).copied().unwrap_or(true);
        if settings::should_notify_threshold(usage, threshold, is_armed) {
            notifications.push((
                "Mochi usage warning".to_string(),
                format!(
                    "{} at {}%",
                    crate::tray::provider_display_name(provider),
                    usage.round() as u8
                ),
            ));
            armed.insert(key, false);
        } else if settings::rearmed_below_threshold(usage, threshold) {
            armed.insert(key, true);
        }
    }

    let enabled: std::collections::HashSet<ProviderId> = settings
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .collect();
    let enabled_snapshots: Vec<UsageSnapshot> = snapshots
        .iter()
        .filter(|snapshot| enabled.contains(&snapshot.provider))
        .cloned()
        .collect();
    let overall_usage = crate::tray::aggregate_used_percent(&enabled_snapshots) as f64;
    let overall_threshold = settings::clamp_warn_percent(settings.usage_warn_percent);
    let overall_armed = armed.get("overall").copied().unwrap_or(true);
    if settings::should_notify_threshold(overall_usage, overall_threshold, overall_armed) {
        notifications.push((
            "Mochi usage warning".to_string(),
            format!("Overall at {}%", overall_usage.round() as u8),
        ));
        armed.insert("overall".to_string(), false);
    } else if settings::rearmed_below_threshold(overall_usage, overall_threshold) {
        armed.insert("overall".to_string(), true);
    }

    notifications
}

/// Evaluate thresholds after a successful refresh and send one-shot notes.
/// The master `show_notifications` toggle still applies inside `send_notification`.
pub fn notify_threshold_crossings(
    app: &AppHandle,
    settings: &MochiSettings,
    snapshots: &[UsageSnapshot],
) {
    let Ok(mut armed) = threshold_armed().lock() else {
        return;
    };
    for (title, body) in evaluate_threshold_notifications(settings, snapshots, &mut armed) {
        send_notification(app, &title, &body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::settings::MochiSettings;

    fn snapshot(provider: ProviderId, used_percent: f32) -> UsageSnapshot {
        UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", used_percent, None),
            None,
            "2026-05-20T12:00:00Z",
            "test",
        )
    }

    fn claude_enabled() -> MochiSettings {
        MochiSettings {
            enabled_providers: vec!["claude".into()],
            ..MochiSettings::default()
        }
    }

    #[test]
    fn disabled_master_toggle_suppresses_notification() {
        assert!(!should_send_notification(false));
    }

    #[test]
    fn enabled_master_toggle_allows_notification() {
        assert!(should_send_notification(true));
    }

    #[test]
    fn threshold_crossing_fires_once_then_disarms() {
        let settings = claude_enabled();
        let over = snapshot(ProviderId::Claude, 85.0);
        let mut armed = HashMap::new();

        let first =
            evaluate_threshold_notifications(&settings, std::slice::from_ref(&over), &mut armed);
        assert!(first.iter().any(|(_, body)| body == "Claude at 85%"));

        let second = evaluate_threshold_notifications(&settings, &[over], &mut armed);
        assert!(second.is_empty());
    }

    #[test]
    fn dropping_below_threshold_rearms() {
        let settings = claude_enabled();
        let over = snapshot(ProviderId::Claude, 85.0);
        let under = snapshot(ProviderId::Claude, 50.0);
        let mut armed = HashMap::new();

        assert!(!evaluate_threshold_notifications(
            &settings,
            std::slice::from_ref(&over),
            &mut armed
        )
        .is_empty());
        assert!(evaluate_threshold_notifications(&settings, &[under], &mut armed).is_empty());
        assert!(
            !evaluate_threshold_notifications(&settings, &[over], &mut armed).is_empty(),
            "crossing again after a dip below must re-fire"
        );
    }

    #[test]
    fn provider_override_replaces_global_threshold() {
        let mut settings = claude_enabled();
        settings.provider_configs.insert(
            "claude".to_string(),
            crate::settings::ProviderConfig {
                warn_percent: Some(90),
                ..Default::default()
            },
        );
        let at_85 = snapshot(ProviderId::Claude, 85.0);
        let mut armed = HashMap::new();

        let fired = evaluate_threshold_notifications(&settings, &[at_85], &mut armed);
        assert!(
            !fired.iter().any(|(_, body)| body.starts_with("Claude at")),
            "provider override 90 must suppress the per-provider note at 85%"
        );
    }
}
