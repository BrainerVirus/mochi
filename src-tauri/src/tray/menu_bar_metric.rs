//! CodexBar `MenuBarMetricWindowResolver` automatic-mode parity for tray % display.
//!
//! Reference: `CodexBar/Sources/CodexBar/MenuBarMetricWindowResolver.swift`

use crate::core::models::{ProviderCostSnapshot, ProviderId, UsageSnapshot, UsageWindow};

/// Remaining % for tray title/icon for one provider snapshot (automatic metric preference).
pub fn tray_remaining_percent(snapshot: &UsageSnapshot) -> u8 {
    if snapshot.provider == ProviderId::Claude
        && should_use_claude_spend_limit(snapshot.provider_cost.as_ref(), snapshot)
    {
        return extra_usage_remaining_percent(snapshot.provider_cost.as_ref());
    }

    automatic_menu_bar_window(snapshot)
        .map(window_remaining_percent)
        .unwrap_or(100)
}

pub fn automatic_menu_bar_window(snapshot: &UsageSnapshot) -> Option<&UsageWindow> {
    match snapshot.provider {
        ProviderId::Antigravity => first_window(
            snapshot,
            [
                WindowLane::Primary,
                WindowLane::Secondary,
                WindowLane::Tertiary,
            ],
        ),
        ProviderId::Zai => {
            most_constrained_pair(Some(&snapshot.primary), snapshot.tertiary.as_ref())
                .or(snapshot.secondary.as_ref())
        }
        ProviderId::Factory => snapshot.secondary.as_ref().or(Some(&snapshot.primary)),
        ProviderId::Copilot => Some(copilot_automatic_window(snapshot)),
        ProviderId::Cursor => most_constrained_all(snapshot),
        ProviderId::Claude => Some(&snapshot.primary),
        // OpenCode family exposes 5-hour / weekly / monthly lanes; the tightest quota
        // matches Cursor-style automatic resolution and reflects binding weekly/monthly limits.
        ProviderId::OpenCode | ProviderId::OpenCodeGo => most_constrained_all(snapshot),
        _ => Some(&snapshot.primary),
    }
}

fn window_remaining_percent(window: &UsageWindow) -> u8 {
    (100.0 - window.used_percent).round().clamp(0.0, 100.0) as u8
}

#[derive(Clone, Copy)]
enum WindowLane {
    Primary,
    Secondary,
    Tertiary,
}

fn first_window(snapshot: &UsageSnapshot, lanes: [WindowLane; 3]) -> Option<&UsageWindow> {
    lanes
        .iter()
        .find_map(|lane| window_for_lane(snapshot, *lane))
}

fn window_for_lane(snapshot: &UsageSnapshot, lane: WindowLane) -> Option<&UsageWindow> {
    match lane {
        WindowLane::Primary => Some(&snapshot.primary),
        WindowLane::Secondary => snapshot.secondary.as_ref(),
        WindowLane::Tertiary => snapshot.tertiary.as_ref(),
    }
}

fn most_constrained_all(snapshot: &UsageSnapshot) -> Option<&UsageWindow> {
    let mut windows = vec![&snapshot.primary];
    if let Some(secondary) = &snapshot.secondary {
        windows.push(secondary);
    }
    if let Some(tertiary) = &snapshot.tertiary {
        windows.push(tertiary);
    }
    most_constrained_slice(&windows)
}

fn most_constrained_pair<'a>(
    left: Option<&'a UsageWindow>,
    right: Option<&'a UsageWindow>,
) -> Option<&'a UsageWindow> {
    match (left, right) {
        (Some(left), Some(right)) => {
            if left.used_percent >= right.used_percent {
                Some(left)
            } else {
                Some(right)
            }
        }
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn most_constrained_slice<'a>(windows: &[&'a UsageWindow]) -> Option<&'a UsageWindow> {
    windows.iter().copied().max_by(|left, right| {
        left.used_percent
            .partial_cmp(&right.used_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn copilot_automatic_window(snapshot: &UsageSnapshot) -> &UsageWindow {
    if let Some(secondary) = &snapshot.secondary {
        if snapshot.primary.used_percent >= secondary.used_percent {
            &snapshot.primary
        } else {
            secondary
        }
    } else {
        &snapshot.primary
    }
}

fn should_use_claude_spend_limit(
    provider_cost: Option<&ProviderCostSnapshot>,
    snapshot: &UsageSnapshot,
) -> bool {
    let Some(cost) = provider_cost else {
        return false;
    };
    if cost.limit <= 0.0 {
        return false;
    }
    if snapshot.secondary.is_some() || snapshot.tertiary.is_some() {
        return false;
    }
    if snapshot.primary.used_percent != 0.0 {
        return false;
    }
    snapshot.primary.label == "Session" && snapshot.primary.resets_at.is_none()
}

fn extra_usage_remaining_percent(cost: Option<&ProviderCostSnapshot>) -> u8 {
    let Some(cost) = cost else {
        return 100;
    };
    let used_percent = ((cost.used / cost.limit) * 100.0).clamp(0.0, 100.0);
    (100.0 - used_percent).round().clamp(0.0, 100.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderCostSnapshot, UsageSnapshot};

    fn window(label: &str, used_percent: f32) -> UsageWindow {
        UsageWindow::new(label, used_percent, None)
    }

    fn snapshot(
        provider: ProviderId,
        primary: UsageWindow,
        secondary: Option<UsageWindow>,
        tertiary: Option<UsageWindow>,
    ) -> UsageSnapshot {
        let mut snapshot =
            UsageSnapshot::new(provider, primary, secondary, "2026-05-22T12:00:00Z", "test");
        if let Some(tertiary) = tertiary {
            snapshot = snapshot.with_tertiary(tertiary);
        }
        snapshot
    }

    #[test]
    fn codex_automatic_uses_primary_session_window() {
        let snap = snapshot(
            ProviderId::Codex,
            window("Session", 8.0),
            Some(window("Weekly", 40.0)),
            None,
        );
        assert_eq!(tray_remaining_percent(&snap), 92);
    }

    #[test]
    fn cursor_automatic_uses_most_constrained_window() {
        let snap = snapshot(
            ProviderId::Cursor,
            window("Total", 10.0),
            Some(window("Auto + Composer", 20.0)),
            Some(window("API", 95.0)),
        );
        assert_eq!(tray_remaining_percent(&snap), 5);
    }

    #[test]
    fn copilot_automatic_prefers_higher_used_lane() {
        let snap = snapshot(
            ProviderId::Copilot,
            window("Premium", 60.0),
            Some(window("Chat", 40.0)),
            None,
        );
        assert_eq!(tray_remaining_percent(&snap), 40);
    }

    #[test]
    fn factory_automatic_prefers_secondary_window() {
        let snap = snapshot(
            ProviderId::Factory,
            window("Primary", 10.0),
            Some(window("Secondary", 80.0)),
            None,
        );
        assert_eq!(tray_remaining_percent(&snap), 20);
    }

    #[test]
    fn zai_automatic_compares_primary_and_tertiary_before_secondary() {
        let snap = snapshot(
            ProviderId::Zai,
            window("5 hours", 12.0),
            Some(window("Monthly", 10.0)),
            Some(window("Token", 92.0)),
        );
        assert_eq!(tray_remaining_percent(&snap), 8);
    }

    #[test]
    fn opencode_go_automatic_uses_tightest_rate_window() {
        let snap = snapshot(
            ProviderId::OpenCodeGo,
            window("5-hour", 0.0),
            Some(window("Weekly", 99.0)),
            Some(window("Monthly", 12.0)),
        );
        assert_eq!(tray_remaining_percent(&snap), 1);
    }

    #[test]
    fn opencode_automatic_uses_tightest_rate_window() {
        let snap = snapshot(
            ProviderId::OpenCode,
            window("5-hour", 0.0),
            Some(window("Weekly", 75.0)),
            None,
        );
        assert_eq!(tray_remaining_percent(&snap), 25);
    }

    #[test]
    fn claude_automatic_uses_session_when_spend_limit_not_placeholder() {
        let snap = snapshot(
            ProviderId::Claude,
            window("Session", 42.0),
            Some(window("Weekly", 10.0)),
            None,
        );
        assert_eq!(tray_remaining_percent(&snap), 58);
    }

    #[test]
    fn claude_automatic_uses_enterprise_spend_limit_when_session_is_placeholder() {
        let mut snap = snapshot(ProviderId::Claude, window("Session", 0.0), None, None);
        snap.provider_cost = Some(ProviderCostSnapshot::new(
            67.03,
            1000.0,
            "USD",
            Some("Monthly".to_string()),
            None,
        ));
        assert_eq!(tray_remaining_percent(&snap), 93);
    }

    #[test]
    fn gemini_automatic_uses_primary_window() {
        let snap = snapshot(
            ProviderId::Gemini,
            window("Pro", 35.0),
            Some(window("Flash", 10.0)),
            Some(window("Flash Lite", 5.0)),
        );
        assert_eq!(tray_remaining_percent(&snap), 65);
    }

    #[test]
    fn antigravity_automatic_follows_primary_secondary_tertiary_order() {
        let snap = snapshot(
            ProviderId::Antigravity,
            window("Primary", 20.0),
            Some(window("Secondary", 30.0)),
            Some(window("Tertiary", 40.0)),
        );
        assert_eq!(tray_remaining_percent(&snap), 80);
    }
}
