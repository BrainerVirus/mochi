use crate::core::models::UsageSnapshot;

pub const TRAY_ID: &str = "mochi-tray";

const WARNING_THRESHOLD: u8 = 60;
const CRITICAL_THRESHOLD: u8 = 85;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayUsageTone {
    Normal,
    Warning,
    Critical,
}

pub fn aggregate_used_percent(snapshots: &[UsageSnapshot]) -> u8 {
    snapshots
        .iter()
        .map(|snapshot| snapshot.primary.used_percent.round() as u8)
        .max()
        .unwrap_or(0)
        .min(100)
}

pub fn tray_usage_tone(used_percent: u8) -> TrayUsageTone {
    if used_percent >= CRITICAL_THRESHOLD {
        TrayUsageTone::Critical
    } else if used_percent >= WARNING_THRESHOLD {
        TrayUsageTone::Warning
    } else {
        TrayUsageTone::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageWindow};

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
    fn aggregate_used_percent_returns_max_primary_usage() {
        let snapshots = vec![snapshot(12.0), snapshot(67.0), snapshot(41.0)];
        assert_eq!(aggregate_used_percent(&snapshots), 67);
    }

    #[test]
    fn aggregate_used_percent_returns_zero_when_empty() {
        assert_eq!(aggregate_used_percent(&[]), 0);
    }

    #[test]
    fn tray_usage_tone_matches_design_thresholds() {
        assert_eq!(tray_usage_tone(59), TrayUsageTone::Normal);
        assert_eq!(tray_usage_tone(60), TrayUsageTone::Warning);
        assert_eq!(tray_usage_tone(84), TrayUsageTone::Warning);
        assert_eq!(tray_usage_tone(85), TrayUsageTone::Critical);
    }
}
