use crate::core::models::ProviderId;
use crate::core::usage_state::ProviderUsageState;
use crate::tray::provider_display_name;

#[derive(Debug, Clone, PartialEq)]
pub struct CostEntry {
    pub provider: ProviderId,
    pub used: f64,
    pub limit: f64,
    pub currency_code: String,
    pub period: String,
}

pub fn format_cost_text(entries: &[CostEntry], days: u16) -> String {
    if entries.is_empty() {
        return format!("No cost data in the last {days} days.");
    }
    entries
        .iter()
        .map(|entry| {
            format!(
                "{} ${:.2} / ${:.2} {} ({})",
                provider_display_name(entry.provider),
                entry.used,
                entry.limit,
                entry.currency_code,
                entry.period
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn cost_entries_from_states(states: &[ProviderUsageState]) -> Vec<CostEntry> {
    states
        .iter()
        .filter_map(|state| {
            let snapshot = state.snapshot.as_ref()?;
            let cost = snapshot.provider_cost.as_ref()?;
            Some(CostEntry {
                provider: snapshot.provider,
                used: cost.used,
                limit: cost.limit,
                currency_code: cost.currency_code.clone(),
                period: cost
                    .period
                    .clone()
                    .unwrap_or_else(|| "current period".to_string()),
            })
        })
        .collect()
}

pub fn load_cost_entries(_days: u16) -> anyhow::Result<Vec<CostEntry>> {
    // Same usage.sqlite3 store the other CLI helpers read (see `cli_usage_states`
    // in lib.rs); spend rides on the latest cached snapshots, so `days` only
    // shapes the empty message in `format_cost_text`.
    let states = crate::cli_usage_states(None, false)?;
    Ok(cost_entries_from_states(&states))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderCostSnapshot, ProviderId, UsageSnapshot, UsageWindow};

    #[test]
    fn cost_line_shows_used_vs_limit() {
        let entries = vec![CostEntry {
            provider: ProviderId::CommandCode,
            used: 7.54,
            limit: 71.93,
            currency_code: "USD".to_string(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30);
        assert!(output.contains("$7.54 / $71.93"));
        assert!(output.contains("USD"));
    }

    #[test]
    fn cost_empty_reports_range() {
        assert!(format_cost_text(&[], 30).contains("30 days"));
    }

    fn snapshot_with_cost() -> ProviderUsageState {
        let snapshot = UsageSnapshot::new(
            ProviderId::Cursor,
            UsageWindow::new("Session", 10.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        )
        .with_provider_cost(ProviderCostSnapshot::new(
            7.54,
            71.93,
            "USD",
            Some("billing-period".to_string()),
            None,
        ));
        ProviderUsageState::fresh(snapshot)
    }

    #[test]
    fn cost_entries_from_states_extracts_provider_cost() {
        let entries = cost_entries_from_states(&[snapshot_with_cost()]);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].provider, ProviderId::Cursor);
        assert_eq!(entries[0].currency_code, "USD");
        assert_eq!(entries[0].period, "billing-period");
    }

    #[test]
    fn cost_entries_from_states_skips_snapshots_without_cost() {
        let snapshot = UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", 10.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        );
        let entries = cost_entries_from_states(&[ProviderUsageState::fresh(snapshot)]);
        assert!(entries.is_empty());
    }
}
