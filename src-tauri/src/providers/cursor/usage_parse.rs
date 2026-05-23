//! Cursor `usage-summary` JSON mapping.
//!
//! Logic derived from CodexBar `CursorStatusProbe.swift` (MIT).

use serde::Deserialize;

use crate::core::models::{ProviderCostSnapshot, ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::ProviderResult;

#[derive(Debug, Clone, Deserialize)]
pub struct CursorUsageSummary {
    #[serde(rename = "billingCycleEnd", default)]
    pub billing_cycle_end: Option<String>,
    #[serde(rename = "membershipType", default)]
    #[allow(dead_code)]
    pub membership_type: Option<String>,
    #[serde(rename = "individualUsage", default)]
    pub individual_usage: Option<CursorIndividualUsage>,
    #[serde(rename = "teamUsage", default)]
    pub team_usage: Option<CursorTeamUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CursorIndividualUsage {
    pub plan: Option<CursorPlanUsage>,
    #[serde(rename = "onDemand", default)]
    pub on_demand: Option<CursorMeterUsage>,
    pub overall: Option<CursorMeterUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CursorTeamUsage {
    pub pooled: Option<CursorMeterUsage>,
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
    let billing_reset = summary.billing_cycle_end.clone();
    let plan_percent = plan_percent_used(summary);
    let auto_percent = summary
        .individual_usage
        .as_ref()
        .and_then(|usage| usage.plan.as_ref())
        .and_then(|plan| normalize_percent(plan.auto_percent_used));
    let api_percent = summary
        .individual_usage
        .as_ref()
        .and_then(|usage| usage.plan.as_ref())
        .and_then(|plan| normalize_percent(plan.api_percent_used));

    let mut snapshot = UsageSnapshot::new(
        ProviderId::Cursor,
        UsageWindow::new("Total", plan_percent, billing_reset.clone()),
        auto_percent.map(|percent| UsageWindow::new("Auto", percent, billing_reset.clone())),
        updated_at,
        source,
    );

    if let Some(api_percent) = api_percent {
        snapshot = snapshot.with_tertiary(UsageWindow::new("API", api_percent, billing_reset.clone()));
    }

    if let Some(cost) = provider_cost(summary, billing_reset) {
        snapshot = snapshot.with_provider_cost(cost);
    }

    Ok(snapshot)
}

fn provider_cost(summary: &CursorUsageSummary, billing_reset: Option<String>) -> Option<ProviderCostSnapshot> {
    let on_demand = summary.individual_usage.as_ref()?.on_demand.as_ref()?;
    let used_cents = on_demand.used.unwrap_or(0) as f64;
    let limit_cents = on_demand.limit.map(|value| value as f64);
    let used_usd = used_cents / 100.0;
    let limit_usd = limit_cents.unwrap_or(0.0) / 100.0;

    if used_usd <= 0.0 && limit_usd <= 0.0 {
        return None;
    }

    Some(ProviderCostSnapshot::new(
        used_usd,
        limit_usd,
        "USD",
        Some("Monthly".into()),
        billing_reset,
    ))
}

fn plan_percent_used(summary: &CursorUsageSummary) -> f32 {
    let plan = summary
        .individual_usage
        .as_ref()
        .and_then(|usage| usage.plan.as_ref());

    if let Some(total) = plan
        .and_then(|plan| normalize_percent(plan.total_percent_used))
    {
        return total;
    }

    let auto = plan.and_then(|plan| normalize_percent(plan.auto_percent_used));
    let api = plan.and_then(|plan| normalize_percent(plan.api_percent_used));
    if let (Some(auto), Some(api)) = (auto, api) {
        return ((auto + api) / 2.0).clamp(0.0, 100.0);
    }
    if let Some(api) = api {
        return api;
    }
    if let Some(auto) = auto {
        return auto;
    }

    let plan_used = plan.and_then(|plan| plan.used).unwrap_or(0) as f64;
    let plan_limit = plan.and_then(|plan| plan.limit).unwrap_or(0) as f64;
    if plan_limit > 0.0 {
        return ((plan_used / plan_limit) * 100.0).clamp(0.0, 100.0) as f32;
    }

    if let Some(overall) = summary
        .individual_usage
        .as_ref()
        .and_then(|usage| usage.overall.as_ref())
    {
        if let Some(percent) = meter_percent(overall) {
            return percent;
        }
    }

    if let Some(pooled) = summary
        .team_usage
        .as_ref()
        .and_then(|usage| usage.pooled.as_ref())
    {
        if let Some(percent) = meter_percent(pooled) {
            return percent;
        }
    }

    0.0
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
    fn maps_usage_summary_fixture_to_total_auto_and_on_demand_cost() {
        let summary: CursorUsageSummary =
            serde_json::from_str(include_str!("fixtures/usage_summary.json")).expect("json");
        let snapshot = snapshot_from_usage_summary(&summary, "2026-05-22T12:00:00Z", "cursor-web")
            .expect("snapshot");

        assert_eq!(snapshot.primary.label, "Total");
        assert_eq!(snapshot.primary.used_percent, 30.0);
        assert_eq!(
            snapshot.primary.resets_at.as_deref(),
            Some("2025-02-01T00:00:00.000Z")
        );

        let secondary = snapshot.secondary.expect("auto");
        assert_eq!(secondary.label, "Auto");
        assert_eq!(secondary.used_percent, 12.0);
        let tertiary = snapshot.tertiary.expect("api");
        assert_eq!(tertiary.label, "API");
        assert_eq!(tertiary.used_percent, 3.0);

        let cost = snapshot.provider_cost.expect("on-demand cost");
        assert_eq!(cost.used, 5.0);
        assert_eq!(cost.limit, 100.0);
        assert_eq!(cost.period.as_deref(), Some("Monthly"));
    }

    #[test]
    fn maps_live_payload_with_auto_api_and_total_lanes() {
        let summary = CursorUsageSummary {
            billing_cycle_end: Some("2026-04-18T20:45:42.000Z".into()),
            membership_type: Some("pro".into()),
            individual_usage: Some(CursorIndividualUsage {
                plan: Some(CursorPlanUsage {
                    used: Some(86),
                    limit: Some(2000),
                    auto_percent_used: Some(0.36),
                    api_percent_used: Some(0.7111111111111111),
                    total_percent_used: Some(0.441025641025641),
                }),
                on_demand: Some(CursorMeterUsage {
                    used: Some(0),
                    limit: None,
                }),
                overall: None,
            }),
            team_usage: None,
        };

        let snapshot =
            snapshot_from_usage_summary(&summary, "2026-05-22T12:00:00Z", "cursor-web").expect("snapshot");

        assert_eq!(snapshot.primary.label, "Total");
        assert!((snapshot.primary.used_percent - 0.441_025_64).abs() < 1e-6);
        assert!((snapshot.secondary.expect("auto").used_percent - 0.36).abs() < 1e-6);
        assert!((snapshot.tertiary.expect("api").used_percent - 0.711_111_1).abs() < 1e-6);
        assert!(snapshot.provider_cost.is_none());
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
        assert!(snapshot.tertiary.is_none());
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
            team_usage: None,
        };

        assert_eq!(plan_percent_used(&summary), 0.40625);
    }

    #[test]
    fn provider_cost_includes_budget_before_first_spend() {
        let summary = CursorUsageSummary {
            billing_cycle_end: None,
            membership_type: Some("pro".into()),
            individual_usage: Some(CursorIndividualUsage {
                plan: Some(CursorPlanUsage {
                    used: Some(500),
                    limit: Some(5000),
                    auto_percent_used: Some(5.0),
                    api_percent_used: None,
                    total_percent_used: Some(10.0),
                }),
                on_demand: Some(CursorMeterUsage {
                    used: Some(0),
                    limit: Some(7500),
                }),
                overall: None,
            }),
            team_usage: None,
        };

        let snapshot =
            snapshot_from_usage_summary(&summary, "2026-05-22T12:00:00Z", "cursor-web").expect("snapshot");
        let cost = snapshot.provider_cost.expect("budget");
        assert_eq!(cost.used, 0.0);
        assert_eq!(cost.limit, 75.0);
    }
}
