use serde::Deserialize;

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Deserialize)]
struct RateLimitsReadResult {
    #[serde(rename = "rateLimits")]
    rate_limits: Option<RateLimitBucket>,
    #[serde(rename = "rateLimitsByLimitId", default)]
    rate_limits_by_limit_id: std::collections::HashMap<String, RateLimitBucket>,
}

#[derive(Debug, Deserialize)]
struct RateLimitBucket {
    #[serde(rename = "limitId")]
    limit_id: Option<String>,
    primary: Option<RateLimitWindow>,
    secondary: Option<RateLimitWindow>,
}

#[derive(Debug, Deserialize)]
struct RateLimitWindow {
    #[serde(rename = "usedPercent")]
    used_percent: f32,
    #[serde(rename = "windowDurationMins")]
    window_duration_mins: Option<u32>,
    #[serde(rename = "resetsAt")]
    resets_at: Option<i64>,
}

pub fn snapshot_from_rate_limits_result(
    result: &serde_json::Value,
    updated_at: &str,
) -> ProviderResult<UsageSnapshot> {
    let payload: RateLimitsReadResult = serde_json::from_value(result.clone())
        .map_err(|error| ProviderError::Parse(error.to_string()))?;

    let bucket = select_codex_bucket(&payload)
        .ok_or_else(|| ProviderError::Parse("codex rate limits missing".into()))?;

    let primary = bucket
        .primary
        .as_ref()
        .ok_or_else(|| ProviderError::Parse("codex primary rate limit missing".into()))?;

    Ok(UsageSnapshot {
        provider: ProviderId::Codex,
        primary: usage_window_from_rate_limit(primary),
        secondary: bucket.secondary.as_ref().map(usage_window_from_rate_limit),
        updated_at: updated_at.to_string(),
        source: "codex-cli".to_string(),
    })
}

fn select_codex_bucket(payload: &RateLimitsReadResult) -> Option<&RateLimitBucket> {
    if let Some(bucket) = payload.rate_limits_by_limit_id.get("codex") {
        return Some(bucket);
    }

    payload.rate_limits.as_ref().filter(|bucket| {
        bucket
            .limit_id
            .as_deref()
            .is_none_or(|limit_id| limit_id == "codex")
    })
}

fn usage_window_from_rate_limit(window: &RateLimitWindow) -> UsageWindow {
    UsageWindow::new(
        window_label(window.window_duration_mins),
        window.used_percent,
        window.resets_at.and_then(format_unix_timestamp),
    )
}

fn window_label(window_duration_mins: Option<u32>) -> &'static str {
    match window_duration_mins {
        Some(minutes) if minutes <= 300 => "Session",
        Some(minutes) if minutes <= 2_880 => "Daily",
        _ => "Weekly",
    }
}

fn format_unix_timestamp(secs: i64) -> Option<String> {
    let datetime = time::OffsetDateTime::from_unix_timestamp(secs).ok()?;
    datetime
        .format(&time::format_description::well_known::Rfc3339)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_codex_rate_limits_fixture_to_usage_snapshot() {
        let result: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/rate_limits.json"))
                .expect("fixture should parse as json");

        let snapshot =
            snapshot_from_rate_limits_result(&result, "2025-12-04T18:10:22Z").expect("snapshot");

        assert_eq!(snapshot.provider, ProviderId::Codex);
        assert_eq!(snapshot.source, "codex-cli");
        assert_eq!(snapshot.primary.label, "Session");
        assert_eq!(snapshot.primary.used_percent, 25.0);
        assert!(snapshot.primary.resets_at.is_some());
        let secondary = snapshot.secondary.expect("weekly window");
        assert_eq!(secondary.label, "Weekly");
        assert_eq!(secondary.used_percent, 59.0);
    }

    #[test]
    fn rejects_payload_without_codex_bucket() {
        let result = serde_json::json!({ "rateLimitsByLimitId": {} });
        let error = snapshot_from_rate_limits_result(&result, "2025-12-04T18:10:22Z")
            .expect_err("missing bucket should fail");

        assert!(matches!(error, ProviderError::Parse(_)));
    }
}
