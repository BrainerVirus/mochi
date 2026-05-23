use crate::core::models::{ProviderId, UsageSnapshot};
use crate::status::filter_configured_snapshots;
use crate::tray::menu_bar_metric::tray_remaining_percent;

/// Which tray tab drives menu bar icon and title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraySelection {
    Overview,
    Provider(ProviderId),
}

impl TraySelection {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            None | Some("overview") => Self::Overview,
            Some(raw) => ProviderId::parse(raw)
                .map(Self::Provider)
                .unwrap_or(Self::Overview),
        }
    }
}

/// Resolved tray status derived from usage snapshots and selected tab.
#[derive(Debug, Clone, PartialEq)]
pub struct TrayIconPresentation {
    pub selection: TraySelection,
    pub remaining_percent: u8,
    pub title: Option<String>,
    pub tooltip: String,
}

pub fn resolve_tray_presentation(
    snapshots: &[UsageSnapshot],
    selection: TraySelection,
) -> TrayIconPresentation {
    match selection {
        TraySelection::Overview => resolve_overview_presentation(snapshots),
        TraySelection::Provider(provider) => resolve_provider_presentation(snapshots, provider)
            .unwrap_or_else(|| resolve_overview_presentation(snapshots)),
    }
}

fn resolve_overview_presentation(snapshots: &[UsageSnapshot]) -> TrayIconPresentation {
    let configured = filter_configured_snapshots(snapshots);
    if configured.is_empty() {
        return TrayIconPresentation {
            selection: TraySelection::Overview,
            remaining_percent: 100,
            title: None,
            tooltip: "Mochi — no usage data".to_string(),
        };
    }

    let remaining = aggregate_remaining_percent(&configured);
    let title = Some(tray_title_from_remaining(remaining));
    let tooltip = format!("Mochi — Overview · {remaining}% left");

    TrayIconPresentation {
        selection: TraySelection::Overview,
        remaining_percent: remaining,
        title,
        tooltip,
    }
}

fn resolve_provider_presentation(
    snapshots: &[UsageSnapshot],
    provider: ProviderId,
) -> Option<TrayIconPresentation> {
    let snapshot = snapshots.iter().find(|entry| entry.provider == provider)?;
    let remaining = remaining_percent(snapshot);
    let title = Some(tray_title_from_remaining(remaining));
    let tooltip = format!(
        "Mochi — {} · {remaining}% left",
        provider_display_name(provider)
    );

    Some(TrayIconPresentation {
        selection: TraySelection::Provider(provider),
        remaining_percent: remaining,
        title,
        tooltip,
    })
}

/// Prefer Codex when present, otherwise the provider closest to its limit (highest used %).
pub fn pick_tray_snapshot(snapshots: &[UsageSnapshot]) -> Option<&UsageSnapshot> {
    if snapshots.is_empty() {
        return None;
    }

    snapshots
        .iter()
        .find(|snapshot| snapshot.provider == ProviderId::Codex)
        .or_else(|| {
            snapshots.iter().max_by(|left, right| {
                left.primary
                    .used_percent
                    .partial_cmp(&right.primary.used_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
}

/// Global % left across configured providers: arithmetic mean of each provider's
/// tray metric remaining % (CodexBar automatic `MenuBarMetricWindowResolver` per provider).
pub fn aggregate_remaining_percent(snapshots: &[UsageSnapshot]) -> u8 {
    let percents: Vec<u8> = snapshots.iter().map(tray_remaining_percent).collect();
    if percents.is_empty() {
        return 100;
    }

    let sum: u32 = percents.iter().map(|percent| u32::from(*percent)).sum();
    (sum as f64 / percents.len() as f64)
        .round()
        .clamp(0.0, 100.0) as u8
}

pub fn remaining_percent(snapshot: &UsageSnapshot) -> u8 {
    tray_remaining_percent(snapshot)
}

/// Menu bar title beside template icon (CodexBar-style leading space on macOS).
pub fn tray_title_from_remaining(remaining: u8) -> String {
    let label = format!("{remaining}%");
    #[cfg(target_os = "macos")]
    {
        format!(" {label}")
    }
    #[cfg(not(target_os = "macos"))]
    {
        label
    }
}

pub fn provider_display_name(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::Codex => "Codex",
        ProviderId::Claude => "Claude",
        ProviderId::Cursor => "Cursor",
        ProviderId::Gemini => "Gemini",
        ProviderId::Copilot => "Copilot",
        ProviderId::OpenCode => "OpenCode",
        ProviderId::OpenCodeGo => "OpenCode Go",
        ProviderId::Antigravity => "Antigravity",
        ProviderId::Factory => "Factory",
        ProviderId::Zai => "Z.ai",
        ProviderId::Kiro => "Kiro",
        ProviderId::Augment => "Augment",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::UsageWindow;

    fn snapshot(provider: ProviderId, used_percent: f32) -> UsageSnapshot {
        UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", used_percent, None),
            None,
            "2026-05-20T12:00:00Z",
            "test",
        )
    }

    #[test]
    fn tray_selection_parse_accepts_overview_and_providers() {
        assert_eq!(TraySelection::parse(None), TraySelection::Overview);
        assert_eq!(
            TraySelection::parse(Some("overview")),
            TraySelection::Overview
        );
        assert_eq!(
            TraySelection::parse(Some("codex")),
            TraySelection::Provider(ProviderId::Codex)
        );
        assert_eq!(
            TraySelection::parse(Some("unknown")),
            TraySelection::Overview
        );
    }

    #[test]
    fn pick_tray_snapshot_prefers_codex_when_present() {
        let snapshots = vec![
            snapshot(ProviderId::Claude, 90.0),
            snapshot(ProviderId::Codex, 5.0),
        ];
        let picked = pick_tray_snapshot(&snapshots).expect("snapshot");
        assert_eq!(picked.provider, ProviderId::Codex);
    }

    #[test]
    fn pick_tray_snapshot_uses_highest_usage_without_codex() {
        let snapshots = vec![
            snapshot(ProviderId::Claude, 12.0),
            snapshot(ProviderId::Cursor, 67.0),
        ];
        let picked = pick_tray_snapshot(&snapshots).expect("snapshot");
        assert_eq!(picked.provider, ProviderId::Cursor);
    }

    #[test]
    fn aggregate_remaining_percent_averages_configured_providers() {
        let snapshots = vec![
            snapshot(ProviderId::Codex, 5.0),
            snapshot(ProviderId::Cursor, 84.0),
            snapshot(ProviderId::OpenCodeGo, 1.0),
        ];
        assert_eq!(aggregate_remaining_percent(&snapshots), 70);
    }

    #[test]
    fn resolve_overview_presentation_uses_aggregate_remaining() {
        let snapshots = vec![
            snapshot(ProviderId::Claude, 12.0),
            snapshot(ProviderId::Cursor, 67.0),
        ];
        let presentation = resolve_tray_presentation(&snapshots, TraySelection::Overview);
        assert_eq!(presentation.selection, TraySelection::Overview);
        assert_eq!(presentation.remaining_percent, 61);
        assert_eq!(presentation.title, Some(tray_title_from_remaining(61)));
        assert!(presentation.tooltip.contains("Overview"));
    }

    #[test]
    fn resolve_overview_presentation_ignores_unconfigured_static_snapshots() {
        let mut static_cursor = snapshot(ProviderId::Cursor, 99.0);
        static_cursor.updated_at = "1970-01-01T00:00:00Z".to_string();
        static_cursor.source = "Cursor".to_string();

        let snapshots = vec![
            snapshot(ProviderId::Codex, 5.0),
            snapshot(ProviderId::Claude, 50.0),
            static_cursor,
        ];
        let presentation = resolve_tray_presentation(&snapshots, TraySelection::Overview);
        assert_eq!(presentation.remaining_percent, 73);
    }

    #[test]
    fn resolve_provider_presentation_uses_selected_provider_remaining() {
        let snapshots = vec![
            snapshot(ProviderId::Codex, 1.0),
            snapshot(ProviderId::Claude, 50.0),
        ];
        let presentation =
            resolve_tray_presentation(&snapshots, TraySelection::Provider(ProviderId::Codex));
        assert_eq!(presentation.remaining_percent, 99);
        assert!(presentation.tooltip.contains("Codex"));
    }

    #[test]
    fn resolve_opencode_go_provider_presentation_uses_tightest_window() {
        let mut opencode_go = snapshot(ProviderId::OpenCodeGo, 0.0);
        opencode_go.secondary = Some(UsageWindow::new("Weekly", 99.0, None));
        opencode_go.tertiary = Some(UsageWindow::new("Monthly", 12.0, None));

        let snapshots = vec![snapshot(ProviderId::Codex, 1.0), opencode_go];
        let presentation = resolve_tray_presentation(
            &snapshots,
            TraySelection::Provider(ProviderId::OpenCodeGo),
        );
        assert_eq!(presentation.remaining_percent, 1);
        assert!(presentation.tooltip.contains("OpenCode Go"));
    }

    #[test]
    fn resolve_tray_presentation_without_snapshots_uses_fallback() {
        let presentation = resolve_tray_presentation(&[], TraySelection::Overview);
        assert_eq!(presentation.selection, TraySelection::Overview);
        assert_eq!(presentation.title, None);
    }
}
