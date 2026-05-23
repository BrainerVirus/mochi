//! Cursor `usage-summary` JSON mapping.
//!
//! Logic derived from CodexBar `CursorStatusProbe.swift` (MIT).

use serde::Deserialize;

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::ProviderResult;

#[derive(Debug, Clone, Deserialize)]
pub struct CursorUsageSummary {
    #[serde(rename = "billingCycleEnd", default)]
    pub billing_cycle_end: Option<String>,
    #[serde(rename = "membershipType", default)]
    pub membership_type: Option<String>,
    #[serde(rename = "individualUsage", default)]
    pub individual_usage: Option<CursorIndividualUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CursorIndividualUsage {
    pub plan: Option<CursorPlanUsage>,
    #[serde(rename = "onDemand", default)]
    pub on_demand: Option<CursorMeterUsage>,
    pub overall: Option<CursorMeterUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CursorPlanUsage {
    pub used: Option<i64>,
    pub limit: Option<i64>,
    #[serde(rename = "autoPercentUsed", default)]
    pub auto_percent_used: Option<f64>,
    #[serde(rename = "apiPercentUsed", default)]
    pub api_percent_used: Option<f64>,
    #[serde(rename = "totalPercentUsed", default)]
    pub total_percent_used: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CursorMeterUsage {
    pub used: Option<i64>,
    pub limit: Option<i64>,
}

pub fn snapshot_from_usage_summary(
    summary: &CursorUsageSummary,
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    let plan_percent = plan_percent_used(summary);
    let primary = UsageWindow::new(
        plan_label(summary.membership_type.as_deref()),
        plan_percent,
        summary.billing_cycle_end.clone(),
    );

    let secondary = on_demand_window(summary.individual_usage.as_ref());

    Ok(UsageSnapshot::new(
        ProviderId::Cursor,
        primary,
        secondary,
        updated_at,
        source,
    ))
}

fn plan_label(membership_type: Option<&str>) -> &'static str {
    match membership_type {
        Some(value) if value.eq_ignore_ascii_case("enterprise") => "Plan",
        Some(value) if value.eq_ignore_ascii_case("pro") => "Plan",
        Some(value) if value.eq_ignore_ascii_case("hobby") => "Plan",
        _ => "Plan",
    }
}

fn plan_percent_used(summary: &CursorUsageSummary) -> f32 {
    let plan = summary
        .individual_usage
        .as_ref()
        .and_then(|usage| usage.plan.as_ref());

    let Some(plan) = plan else {
        return 0.0;
    };

    if let Some(total) = normalize_percent(plan.total_percent_used) {
        return total;
    }

    let auto = normalize_percent(plan.auto_percent_used);
    let api = normalize_percent(plan.api_percent_used);
    if let (Some(auto), Some(api)) = (auto, api) {
        return ((auto + api) / 2.0).clamp(0.0, 100.0);
    }
    if let Some(api) = api {
        return api;
    }
    if let Some(auto) = auto {
        return auto;
    }

    let used = plan.used.unwrap_or(0) as f64;
    let limit = plan.limit.unwrap_or(0) as f64;
    if limit > 0.0 {
        return ((used / limit) * 100.0).clamp(0.0, 100.0) as f32;
    }

    if let Some(overall) = summary
        .individual_usage
        .as_ref()
        .and_then(|usage| usage.overall.as_ref())
    {
        return meter_percent(overall).unwrap_or(0.0);
    }

    0.0
}

fn on_demand_window(individual: Option<&CursorIndividualUsage>) -> Option<UsageWindow> {
    let on_demand = individual?.on_demand.as_ref()?;
    let percent = meter_percent(on_demand)?;
    Some(UsageWindow::new("On-demand", percent, None))
}

fn meter_percent(meter: &CursorMeterUsage) -> Option<f32> {
    let used = meter.used? as f64;
    let limit = meter.limit? as f64;
    if limit <= 0.0 {
        return None;
    }
    Some(((used / limit) * 100.0).clamp(0.0, 100.0) as f32)
}

fn normalize_percent(value: Option<f64>) -> Option<f32> {
    let value = value?;
    Some(value.clamp(0.0, 100.0) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_usage_summary_fixture() {
        let summary: CursorUsageSummary =
            serde_json::from_str(include_str!("fixtures/usage_summary.json")).expect("json");
        let snapshot = snapshot_from_usage_summary(&summary, "2026-05-22T12:00:00Z", "cursor-web")
            .expect("snapshot");

        assert_eq!(snapshot.primary.used_percent, 30.0);
        assert_eq!(
            snapshot.primary.resets_at.as_deref(),
            Some("2025-02-01T00:00:00.000Z")
        );
        let secondary = snapshot.secondary.expect("on-demand");
        assert_eq!(secondary.label, "On-demand");
        assert_eq!(secondary.used_percent, 5.0);
    }

    #[test]
    fn derives_plan_percent_from_used_and_limit() {
        let summary: CursorUsageSummary =
            serde_json::from_str(include_str!("fixtures/usage_summary_minimal.json"))
                .expect("json");
        let snapshot = snapshot_from_usage_summary(&summary, "2026-05-22T12:00:00Z", "cursor-web")
            .expect("snapshot");

        assert_eq!(snapshot.primary.used_percent, 0.0);
        assert!(snapshot.secondary.is_none());
    }

    #[test]
    fn prefers_total_percent_over_ratio() {
        let summary = CursorUsageSummary {
            billing_cycle_end: None,
            membership_type: Some("enterprise".into()),
            individual_usage: Some(CursorIndividualUsage {
                plan: Some(CursorPlanUsage {
                    used: Some(4900),
                    limit: Some(50000),
                    auto_percent_used: None,
                    api_percent_used: None,
                    total_percent_used: Some(0.40625),
                }),
                on_demand: None,
                overall: None,
            }),
        };

        assert_eq!(plan_percent_used(&summary), 0.40625);
    }
}
