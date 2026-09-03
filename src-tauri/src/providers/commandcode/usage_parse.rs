use crate::core::models::{ProviderCostSnapshot, ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone)]
pub struct WindowLimit {
    pub used_percent: f32,
    pub resets_at: Option<String>,
    pub limited: bool,
}

#[derive(Debug, Clone)]
pub struct CreditsResponse {
    pub monthly_credits_remaining: f64,
    pub five_hour: Option<WindowLimit>,
    pub weekly: Option<WindowLimit>,
    pub monthly: Option<WindowLimit>,
}

#[derive(Debug, Clone)]
pub struct SummaryResponse {
    pub total_tokens: f64,
    pub total_tokens_in: f64,
    pub total_tokens_out: f64,
    pub run_count: u64,
    pub total_cost: f64,
    pub success_rate: f64,
}

pub fn parse_credits(value: &serde_json::Value) -> ProviderResult<CreditsResponse> {
    let credits = value
        .get("credits")
        .ok_or_else(|| ProviderError::Parse("commandcode: missing credits".into()))?;
    let monthly_credits_remaining = credits
        .get("monthlyCredits")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| ProviderError::Parse("commandcode: missing monthlyCredits".into()))?;

    let window = |key: &str| -> Option<WindowLimit> {
        value.get("windowLimits")?.get(key).map(|raw| WindowLimit {
            used_percent: raw
                .get("usedPercent")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0) as f32,
            resets_at: raw
                .get("resetsAt")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            limited: raw
                .get("limited")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        })
    };

    Ok(CreditsResponse {
        monthly_credits_remaining,
        five_hour: window("fiveHour"),
        weekly: window("weekly"),
        monthly: window("monthly"),
    })
}

pub fn parse_summary(value: &serde_json::Value) -> ProviderResult<SummaryResponse> {
    let run_count = value
        .get("totalCount")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| ProviderError::Parse("commandcode: missing totalCount".into()))?;
    let total_tokens = value
        .get("totalTokens")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| ProviderError::Parse("commandcode: missing totalTokens".into()))?;

    Ok(SummaryResponse {
        run_count,
        total_tokens,
        total_tokens_in: value
            .get("totalTokensIn")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        total_tokens_out: value
            .get("totalTokensOut")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        total_cost: value
            .get("totalCost")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        success_rate: value
            .get("successRate")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
    })
}

pub fn snapshot_from_commandcode(
    credits: &CreditsResponse,
    summary: &SummaryResponse,
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    let monthly = credits
        .monthly
        .as_ref()
        .ok_or_else(|| ProviderError::Parse("commandcode: missing monthly window".into()))?;
    let primary = UsageWindow::new("Monthly", monthly.used_percent, monthly.resets_at.clone());
    let secondary = credits
        .weekly
        .as_ref()
        .map(|weekly| UsageWindow::new("Weekly", weekly.used_percent, weekly.resets_at.clone()));
    let extra_windows = credits
        .five_hour
        .as_ref()
        .map(|five| UsageWindow::new("5 hours", five.used_percent, five.resets_at.clone()))
        .into_iter()
        .collect();

    let mut snapshot = UsageSnapshot::new(
        ProviderId::CommandCode,
        primary,
        secondary,
        updated_at,
        source,
    )
    .with_extra_windows(extra_windows);

    let used = summary.total_cost;
    let limit = used + credits.monthly_credits_remaining;
    snapshot = snapshot.with_provider_cost(ProviderCostSnapshot::new(
        used,
        limit,
        "USD",
        Some("billing-period".to_string()),
        monthly.resets_at.clone(),
    ));

    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credits_fixture() -> serde_json::Value {
        serde_json::from_str(include_str!("fixtures/credits.json")).unwrap()
    }

    fn summary_fixture() -> serde_json::Value {
        serde_json::from_str(include_str!("fixtures/summary.json")).unwrap()
    }

    #[test]
    fn parses_window_limits() {
        let credits = parse_credits(&credits_fixture()).expect("parse credits");
        let monthly = credits.monthly.as_ref().expect("monthly window");
        assert_eq!(monthly.used_percent, 64.0);
        assert!(monthly.limited);
        assert_eq!(credits.monthly_credits_remaining, 12.5);
    }

    #[test]
    fn parses_summary_totals() {
        let summary = parse_summary(&summary_fixture()).expect("parse summary");
        assert_eq!(summary.run_count, 6319);
        assert_eq!(summary.total_tokens, 1_308_443_159.0);
    }

    #[test]
    fn builds_snapshot_with_three_windows() {
        let credits = parse_credits(&credits_fixture()).expect("credits");
        let summary = parse_summary(&summary_fixture()).expect("summary");
        let snapshot = snapshot_from_commandcode(
            &credits,
            &summary,
            "2026-09-02T19:00:00Z",
            "commandcode-web",
        )
        .expect("snap");
        assert_eq!(snapshot.provider, ProviderId::CommandCode);
        assert_eq!(snapshot.primary.label, "Monthly");
        assert_eq!(snapshot.primary.used_percent, 64.0);
        let five = snapshot
            .extra_windows
            .iter()
            .find(|w| w.label == "5 hours")
            .expect("5h window");
        assert_eq!(five.used_percent, 0.0);
        let cost = snapshot.provider_cost.as_ref().expect("cost");
        assert_eq!(cost.used, 46.80689824929999);
        assert_eq!(cost.limit, 59.30689824929999);
        assert_eq!(cost.currency_code, "USD");
        assert_eq!(cost.period.as_deref(), Some("billing-period"));
        assert_eq!(cost.resets_at.as_deref(), Some("2026-09-10T00:00:00Z"));
    }

    #[test]
    fn rejects_malformed_credits() {
        let bad = serde_json::json!({ "credits": "nope" });
        assert!(parse_credits(&bad).is_err());
    }
}
