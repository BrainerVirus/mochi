//! Usage JSON mapping for Claude OAuth and Web API responses.
//!
//! Logic derived from CodexBar `ClaudeOAuthUsageFetcher` and `ClaudeWebAPIFetcher` (MIT).

use serde::Deserialize;

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone, Deserialize)]
pub struct UsageWindowPayload {
    pub utilization: Option<f64>,
    #[serde(rename = "resets_at")]
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeUsageResponse {
    pub five_hour: Option<UsageWindowPayload>,
    pub seven_day: Option<UsageWindowPayload>,
    #[serde(default)]
    pub seven_day_oauth_apps: Option<UsageWindowPayload>,
    pub seven_day_sonnet: Option<UsageWindowPayload>,
    pub seven_day_opus: Option<UsageWindowPayload>,
    #[serde(default)]
    pub extra_usage: Option<ExtraUsagePayload>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtraUsagePayload {
    #[serde(rename = "is_enabled")]
    pub is_enabled: Option<bool>,
}

pub fn snapshot_from_usage_response(
    response: &ClaudeUsageResponse,
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    let primary = window_from_payload(response.five_hour.as_ref(), "Session")
        .or_else(|| window_from_payload(response.seven_day.as_ref(), "Weekly"))
        .or_else(|| window_from_payload(response.seven_day_oauth_apps.as_ref(), "Weekly"))
        .or_else(|| window_from_payload(response.seven_day_sonnet.as_ref(), "Weekly"))
        .or_else(|| window_from_payload(response.seven_day_opus.as_ref(), "Weekly"));

    let primary = primary.ok_or_else(|| {
        if response.extra_usage.as_ref().and_then(|e| e.is_enabled) == Some(true) {
            ProviderError::Parse(
                "claude usage has extra_usage only; spend-limit UI not implemented".into(),
            )
        } else {
            ProviderError::Parse("claude usage missing rate windows".into())
        }
    })?;

    let secondary = if primary.label == "Session" {
        window_from_payload(response.seven_day.as_ref(), "Weekly")
            .or_else(|| window_from_payload(response.seven_day_sonnet.as_ref(), "Sonnet weekly"))
            .or_else(|| window_from_payload(response.seven_day_opus.as_ref(), "Opus weekly"))
    } else {
        None
    };

    Ok(UsageSnapshot::new(
        ProviderId::Claude,
        primary,
        secondary,
        updated_at,
        source,
    ))
}

fn window_from_payload(
    payload: Option<&UsageWindowPayload>,
    label: &'static str,
) -> Option<UsageWindow> {
    let payload = payload?;
    let used = payload.utilization?;
    Some(UsageWindow::new(
        label,
        used as f32,
        payload.resets_at.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth_fixture() -> ClaudeUsageResponse {
        serde_json::from_str(include_str!("fixtures/oauth_usage.json")).expect("fixture")
    }

    #[test]
    fn maps_oauth_usage_fixture_to_snapshot() {
        let response = oauth_fixture();
        let snapshot =
            snapshot_from_usage_response(&response, "2026-05-22T12:00:00Z", "claude-oauth")
                .expect("snapshot");

        assert_eq!(snapshot.provider, ProviderId::Claude);
        assert_eq!(snapshot.source, "claude-oauth");
        assert_eq!(snapshot.primary.label, "Session");
        assert_eq!(snapshot.primary.used_percent, 12.5);
        let secondary = snapshot.secondary.expect("weekly");
        assert_eq!(secondary.label, "Weekly");
        assert_eq!(secondary.used_percent, 30.0);
    }

    #[test]
    fn maps_web_usage_fixture_to_snapshot() {
        let response: ClaudeUsageResponse =
            serde_json::from_str(include_str!("fixtures/web_usage.json")).expect("fixture");
        let snapshot =
            snapshot_from_usage_response(&response, "2026-05-22T12:00:00Z", "claude-web")
                .expect("snapshot");

        assert_eq!(snapshot.primary.used_percent, 9.0);
        assert_eq!(snapshot.secondary.expect("weekly").used_percent, 22.0);
    }

    #[test]
    fn enterprise_null_five_hour_falls_back_to_weekly() {
        let response: ClaudeUsageResponse = serde_json::from_str(
            r#"{"five_hour":null,"seven_day":{"utilization":15,"resets_at":"2025-12-30T00:00:00.000Z"}}"#,
        )
        .expect("json");

        let snapshot =
            snapshot_from_usage_response(&response, "2026-05-22T12:00:00Z", "claude-web")
                .expect("snapshot");

        assert_eq!(snapshot.primary.label, "Weekly");
        assert_eq!(snapshot.primary.used_percent, 15.0);
        assert!(snapshot.secondary.is_none());
    }
}
