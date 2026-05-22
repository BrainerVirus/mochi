use serde::Deserialize;
use time::OffsetDateTime;

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone, Deserialize)]
pub struct CodexUsageResponse {
    #[serde(default, rename = "plan_type")]
    pub _plan_type: Option<String>,
    pub rate_limit: Option<RateLimitDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitDetails {
    pub primary_window: Option<WindowSnapshot>,
    pub secondary_window: Option<WindowSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowSnapshot {
    pub used_percent: i32,
    pub reset_at: i64,
    pub limit_window_seconds: i32,
}

pub fn snapshot_from_oauth_usage(
    response: &CodexUsageResponse,
    updated_at: &str,
) -> ProviderResult<UsageSnapshot> {
    let rate_limit = response
        .rate_limit
        .as_ref()
        .ok_or_else(|| ProviderError::Parse("codex oauth usage missing rate_limit".into()))?;

    let primary = rate_limit
        .primary_window
        .as_ref()
        .map(window_to_usage)
        .transpose()?;
    let secondary = rate_limit
        .secondary_window
        .as_ref()
        .map(window_to_usage)
        .transpose()?;

    let (primary, secondary) = match (primary, secondary) {
        (Some(primary), secondary) => (primary, secondary),
        (None, Some(secondary)) => (secondary, None),
        (None, None) => {
            return Err(ProviderError::Parse(
                "codex oauth usage missing rate windows".into(),
            ));
        }
    };

    Ok(UsageSnapshot::new(
        ProviderId::Codex,
        primary,
        secondary,
        updated_at,
        "codex-oauth",
    ))
}

fn window_to_usage(window: &WindowSnapshot) -> ProviderResult<UsageWindow> {
    Ok(UsageWindow::new(
        window_label(window.limit_window_seconds),
        window.used_percent as f32,
        format_unix_timestamp(window.reset_at),
    ))
}

fn window_label(limit_window_seconds: i32) -> &'static str {
    let minutes = limit_window_seconds / 60;
    match minutes {
        m if m <= 300 => "Session",
        m if m <= 2_880 => "Daily",
        _ => "Weekly",
    }
}

fn format_unix_timestamp(secs: i64) -> Option<String> {
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()?
        .format(&time::format_description::well_known::Rfc3339)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_usage_response() -> CodexUsageResponse {
        serde_json::from_str(include_str!("../fixtures/oauth_usage.json")).expect("fixture json")
    }

    #[test]
    fn maps_oauth_usage_fixture_to_snapshot() {
        let response = fixture_usage_response();
        let snapshot =
            snapshot_from_oauth_usage(&response, "2026-05-20T12:00:00Z").expect("snapshot");

        assert_eq!(snapshot.provider, ProviderId::Codex);
        assert_eq!(snapshot.source, "codex-oauth");
        assert_eq!(snapshot.primary.label, "Session");
        assert_eq!(snapshot.primary.used_percent, 22.0);
        let secondary = snapshot.secondary.expect("weekly window");
        assert_eq!(secondary.label, "Weekly");
        assert_eq!(secondary.used_percent, 43.0);
    }
}
