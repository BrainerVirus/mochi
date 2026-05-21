use crate::core::models::{ProviderId, UsageSnapshot};

/// Resolved tray status derived from usage snapshots (CodexBar-style: provider + % remaining).
#[derive(Debug, Clone, PartialEq)]
pub struct TrayIconPresentation {
    pub provider: Option<ProviderId>,
    pub remaining_percent: u8,
    pub title: Option<String>,
    pub tooltip: String,
}

pub fn resolve_tray_presentation(snapshots: &[UsageSnapshot]) -> TrayIconPresentation {
    let Some(snapshot) = pick_tray_snapshot(snapshots) else {
        return TrayIconPresentation {
            provider: None,
            remaining_percent: 100,
            title: None,
            tooltip: "Mochi — no usage data".to_string(),
        };
    };

    let remaining = remaining_percent(snapshot);
    let provider = snapshot.provider;
    let title = Some(format!("{remaining}%"));
    let tooltip = format!(
        "Mochi — {} · {remaining}% left",
        provider_display_name(provider)
    );

    TrayIconPresentation {
        provider: Some(provider),
        remaining_percent: remaining,
        title,
        tooltip,
    }
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

pub fn remaining_percent(snapshot: &UsageSnapshot) -> u8 {
    snapshot
        .primary
        .remaining_percent
        .round()
        .clamp(0.0, 100.0) as u8
}

pub fn provider_display_name(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::Codex => "Codex",
        ProviderId::Claude => "Claude",
        ProviderId::Cursor => "Cursor",
        ProviderId::Gemini => "Gemini",
        ProviderId::Copilot => "Copilot",
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
        UsageSnapshot {
            provider,
            primary: UsageWindow::new("Session", used_percent, None),
            secondary: None,
            updated_at: "1970-01-01T00:00:00Z".to_string(),
            source: "test".to_string(),
        }
    }

    #[test]
    fn pick_tray_snapshot_prefers_codex_when_present() {
        let snapshots = vec![snapshot(ProviderId::Claude, 90.0), snapshot(ProviderId::Codex, 5.0)];
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
    fn resolve_tray_presentation_formats_remaining_percent() {
        let snapshots = vec![snapshot(ProviderId::Codex, 1.0)];
        let presentation = resolve_tray_presentation(&snapshots);
        assert_eq!(presentation.remaining_percent, 99);
        assert_eq!(presentation.title.as_deref(), Some("99%"));
        assert!(presentation.tooltip.contains("Codex"));
        assert!(presentation.tooltip.contains("99% left"));
    }

    #[test]
    fn resolve_tray_presentation_without_snapshots_uses_fallback() {
        let presentation = resolve_tray_presentation(&[]);
        assert_eq!(presentation.provider, None);
        assert_eq!(presentation.title, None);
    }
}
