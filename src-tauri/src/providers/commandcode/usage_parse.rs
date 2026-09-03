use crate::core::models::{ProviderCostSnapshot, ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct WindowLimit {
    pub used_percent: f32,
    pub resets_at: Option<String>,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub total_tokens: f64,
    #[allow(dead_code)]
    pub total_tokens_in: f64,
    #[allow(dead_code)]
    pub total_tokens_out: f64,
    #[allow(dead_code)]
    pub run_count: u64,
    pub total_cost: f64,
    #[allow(dead_code)]
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
        let raw = value.get("windowLimits")?.get(key)?;
        let used = raw.get("used").and_then(serde_json::Value::as_f64)?;
        let cap = raw
            .get("cap")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0);
        Some(WindowLimit {
            used_percent: if cap > 0.0 {
                (used / cap * 100.0) as f32
            } else {
                0.0
            },
            resets_at: raw
                .get("resetAt")
                .and_then(serde_json::Value::as_i64)
                .and_then(reset_at_to_rfc3339),
            limited: raw
                .get("exceeded")
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

fn reset_at_to_rfc3339(epoch_ms: i64) -> Option<String> {
    OffsetDateTime::from_unix_timestamp_nanos(epoch_ms as i128 * 1_000_000)
        .ok()?
        .format(&Rfc3339)
        .ok()
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
    let weekly = credits
        .weekly
        .as_ref()
        .ok_or_else(|| ProviderError::Parse("commandcode: missing weekly window".into()))?;
    let primary = UsageWindow::new("Weekly", weekly.used_percent, weekly.resets_at.clone());
    let secondary = credits
        .five_hour
        .as_ref()
        .map(|five| UsageWindow::new("5 hours", five.used_percent, five.resets_at.clone()));
    let extra_windows: Vec<UsageWindow> = credits
        .monthly
        .as_ref()
        .map(|monthly| UsageWindow::new("Monthly", monthly.used_percent, monthly.resets_at.clone()))
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
        weekly.resets_at.clone(),
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
        let five = credits.five_hour.as_ref().expect("5h window");
        assert!((five.used_percent - 11.041_829).abs() < 0.001);
        assert_eq!(five.resets_at.as_deref(), Some("2026-09-03T18:50:03.481Z"));
        let weekly = credits.weekly.as_ref().expect("weekly window");
        assert!((weekly.used_percent - 28.468_555).abs() < 0.001);
        assert_eq!(
            weekly.resets_at.as_deref(),
            Some("2026-09-09T18:35:06.872Z")
        );
        assert!(credits.monthly.is_none());
        assert_eq!(credits.monthly_credits_remaining, 15.2591649987);
    }

    #[test]
    fn parses_summary_totals() {
        let summary = parse_summary(&summary_fixture()).expect("parse summary");
        assert_eq!(summary.run_count, 7890);
        assert_eq!(summary.total_tokens, 1_461_198_733.0);
    }

    #[test]
    fn builds_snapshot_with_two_windows() {
        let credits = parse_credits(&credits_fixture()).expect("credits");
        let summary = parse_summary(&summary_fixture()).expect("summary");
        let snapshot = snapshot_from_commandcode(
            &credits,
            &summary,
            "2026-09-03T16:00:00Z",
            "commandcode-web",
        )
        .expect("snap");
        assert_eq!(snapshot.provider, ProviderId::CommandCode);
        assert_eq!(snapshot.primary.label, "Weekly");
        assert!((snapshot.primary.used_percent - 28.468_555).abs() < 0.001);
        let five = snapshot.secondary.as_ref().expect("5h window");
        assert!((five.used_percent - 11.041_829).abs() < 0.001);
        let cost = snapshot.provider_cost.as_ref().expect("cost");
        assert!((cost.used - 56.74708297929999).abs() < 0.001);
        assert!((cost.limit - 72.00624797799999).abs() < 0.001);
        assert_eq!(cost.currency_code, "USD");
        assert_eq!(cost.period.as_deref(), Some("billing-period"));
        assert_eq!(cost.resets_at.as_deref(), Some("2026-09-09T18:35:06.872Z"));
    }

    #[test]
    fn rejects_malformed_credits() {
        let bad = serde_json::json!({ "credits": "nope" });
        assert!(parse_credits(&bad).is_err());
    }
}
