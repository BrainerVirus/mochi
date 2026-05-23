//! HTTP client for Cursor web usage API.
//!
//! Derived from CodexBar `CursorStatusProbe.swift` (MIT).

use std::time::Duration;

use async_trait::async_trait;

use super::credentials::resolve_manual_cookie;
use super::usage_parse::CursorUsageSummary;
use crate::core::provider::{ProviderError, ProviderResult};

const BASE_URL: &str = "https://cursor.com";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait CursorWebClient: Send + Sync {
    async fn resolve_cookie(&self) -> ProviderResult<String>;
    async fn fetch_usage_summary(&self, cookie_header: &str) -> ProviderResult<CursorUsageSummary>;
}

pub struct HttpCursorWebClient {
    http: reqwest::Client,
}

impl HttpCursorWebClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { http }
    }
}

impl Default for HttpCursorWebClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CursorWebClient for HttpCursorWebClient {
    async fn resolve_cookie(&self) -> ProviderResult<String> {
        resolve_manual_cookie()?.ok_or(ProviderError::NotConfigured)
    }

    async fn fetch_usage_summary(&self, cookie_header: &str) -> ProviderResult<CursorUsageSummary> {
        let url = format!("{BASE_URL}/api/usage-summary");
        let response = self
            .http
            .get(url)
            .header("Accept", "application/json")
            .header("Cookie", cookie_header)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let data = response
            .bytes()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        match status.as_u16() {
            200 => serde_json::from_slice(&data)
                .map_err(|error| ProviderError::Parse(error.to_string())),
            401 | 403 => Err(ProviderError::Auth(
                "cursor session unauthorized or expired".into(),
            )),
            code => Err(ProviderError::Fetch(format!(
                "cursor usage-summary request failed: HTTP {code}"
            ))),
        }
    }
}

pub async fn fetch_snapshot_with_client(
    client: &dyn CursorWebClient,
    updated_at: &str,
) -> ProviderResult<crate::core::models::UsageSnapshot> {
    let cookie = client.resolve_cookie().await?;
    let summary = client.fetch_usage_summary(&cookie).await?;
    super::usage_parse::snapshot_from_usage_summary(&summary, updated_at, "cursor-web")
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockCursorWebClient {
        summary: ProviderResult<CursorUsageSummary>,
    }

    #[async_trait]
    impl CursorWebClient for MockCursorWebClient {
        async fn resolve_cookie(&self) -> ProviderResult<String> {
            Ok("WorkosCursorSessionToken=test".into())
        }

        async fn fetch_usage_summary(
            &self,
            _cookie_header: &str,
        ) -> ProviderResult<CursorUsageSummary> {
            match &self.summary {
                Ok(summary) => Ok(summary.clone()),
                Err(ProviderError::NotConfigured) => Err(ProviderError::NotConfigured),
                Err(ProviderError::Auth(message)) => Err(ProviderError::Auth(message.clone())),
                Err(ProviderError::Timeout) => Err(ProviderError::Timeout),
                Err(ProviderError::Parse(message)) => Err(ProviderError::Parse(message.clone())),
                Err(ProviderError::Fetch(message)) => Err(ProviderError::Fetch(message.clone())),
            }
        }
    }

    #[tokio::test]
    async fn fetch_snapshot_with_client_maps_fixture() {
        let summary: CursorUsageSummary =
            serde_json::from_str(include_str!("fixtures/usage_summary.json")).expect("json");
        let client = MockCursorWebClient {
            summary: Ok(summary),
        };

        let snapshot = fetch_snapshot_with_client(&client, "2026-05-22T12:00:00Z")
            .await
            .expect("snapshot");

        assert_eq!(snapshot.source, "cursor-web");
        assert_eq!(snapshot.primary.used_percent, 30.0);
    }
}
