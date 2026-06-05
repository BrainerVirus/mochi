use serde::Serialize;

use crate::core::usage_state::ProviderUsageState;
use crate::tray::provider_display_name;

#[derive(Debug, Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: String,
    pub class: String,
    pub percentage: u8,
}

pub fn waybar_output(label: &str, percentage: u8) -> WaybarOutput {
    let class = match percentage {
        0..=69 => "ok",
        70..=89 => "warning",
        _ => "critical",
    };

    WaybarOutput {
        text: format!("Mochi {percentage}%"),
        tooltip: label.to_string(),
        class: class.to_string(),
        percentage,
    }
}

pub fn format_output(format: &str, percentage: u8, label: &str) -> String {
    match format {
        "waybar" => serde_json::to_string(&waybar_output(label, percentage)).unwrap_or_default(),
        "json" => serde_json::to_string(&waybar_output(label, percentage)).unwrap_or_default(),
        "text" => format!("Mochi {percentage}% ({label})"),
        _ => format!("unsupported format: {format}"),
    }
}

pub fn format_output_from_states(format: &str, states: &[ProviderUsageState]) -> String {
    let Some(snapshot) = states.iter().find_map(|state| state.snapshot.as_ref()) else {
        return format_output(format, 0, "No cached usage");
    };

    format_output(
        format,
        snapshot.primary.used_percent.round() as u8,
        provider_display_name(snapshot.provider),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waybar_marks_critical_usage() {
        let output = waybar_output("Claude", 95);
        assert_eq!(output.class, "critical");
    }

    #[test]
    fn status_bar_formats_from_cached_state() {
        let snapshot = crate::core::models::UsageSnapshot::new(
            crate::core::models::ProviderId::Claude,
            crate::core::models::UsageWindow::new("Session", 72.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        );
        let output = format_output_from_states(
            "text",
            &[crate::core::usage_state::ProviderUsageState::fresh(snapshot)],
        );

        assert_eq!(output, "Mochi 72% (Claude)");
    }
}
