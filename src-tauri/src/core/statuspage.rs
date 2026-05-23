//! Statuspage.io polling (CodexBar-derived, MIT).
//! Reference: CodexBar `Sources/CodexBarCLI/CLIPayloads.swift`.

use serde::Deserialize;
use time::OffsetDateTime;

use crate::core::models::{ProviderStatus, StatusIndicator};
use crate::core::provider::{ProviderError, ProviderResult};

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Debug, Deserialize)]
struct StatuspageResponse {
    page: Option<StatuspagePage>,
    status: StatuspageStatus,
}

#[derive(Debug, Deserialize)]
struct StatuspagePage {
    #[serde(rename = "updated_at")]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StatuspageStatus {
    indicator: String,
    description: Option<String>,
}

pub async fn fetch_status(base_url: &str) -> ProviderResult<ProviderStatus> {
    let api_url = format!("{}/api/v2/status.json", base_url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;

    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|error| ProviderError::Fetch(format!("statuspage request failed: {error}")))?;

    if !response.status().is_success() {
        return Err(ProviderError::Fetch(format!(
            "statuspage returned HTTP {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|error| ProviderError::Fetch(format!("statuspage read failed: {error}")))?;

    parse_status_response(&body, base_url)
}

pub fn parse_status_response(body: &str, base_url: &str) -> ProviderResult<ProviderStatus> {
    let payload: StatuspageResponse =
        serde_json::from_str(body).map_err(|error| ProviderError::Parse(error.to_string()))?;

    Ok(ProviderStatus {
        indicator: parse_indicator(&payload.status.indicator),
        description: payload.status.description,
        updated_at: payload
            .page
            .and_then(|page| page.updated_at)
            .or_else(|| Some(current_timestamp())),
        url: Some(base_url.trim_end_matches('/').to_string()),
    })
}

fn parse_indicator(raw: &str) -> StatusIndicator {
    match raw {
        "none" => StatusIndicator::None,
        "minor" => StatusIndicator::Minor,
        "major" => StatusIndicator::Major,
        "critical" => StatusIndicator::Critical,
        "maintenance" => StatusIndicator::Maintenance,
        _ => StatusIndicator::Unknown,
    }
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_statuspage_fixture_maps_indicator_and_description() {
        let body = include_str!("../providers/codex/fixtures/statuspage.json");
        let status = parse_status_response(body, "https://status.openai.com").expect("status");

        assert_eq!(status.indicator, StatusIndicator::Minor);
        assert_eq!(
            status.description.as_deref(),
            Some("Elevated API error rates")
        );
        assert_eq!(status.url.as_deref(), Some("https://status.openai.com"));
    }

    #[test]
    fn parse_indicator_unknown_for_unrecognized_values() {
        assert_eq!(parse_indicator("investigating"), StatusIndicator::Unknown);
    }
}
