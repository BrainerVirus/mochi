use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use super::usage_parse::{parse_quota_response, snapshot_from_limits};
use crate::core::models::UsageSnapshot;
use crate::core::provider::{ProviderError, ProviderResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait ZaiQuotaClient: Send + Sync {
    async fn fetch_usage(
        &self,
        api_key: &str,
        quota_url: &str,
        updated_at: &str,
        source: &str,
    ) -> ProviderResult<UsageSnapshot>;
}

pub struct HttpZaiQuotaClient {
    http: reqwest::Client,
}

impl HttpZaiQuotaClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }
}

impl Default for HttpZaiQuotaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ZaiQuotaClient for HttpZaiQuotaClient {
    async fn fetch_usage(
        &self,
        api_key: &str,
        quota_url: &str,
        updated_at: &str,
        source: &str,
    ) -> ProviderResult<UsageSnapshot> {
        let response = self
            .http
            .get(quota_url)
            .header(AUTHORIZATION, format!("Bearer {api_key}"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        if !status.is_success() {
            return Err(ProviderError::Fetch(format!(
                "z.ai quota HTTP {}: {body}",
                status.as_u16()
            )));
        }

        if body.trim().is_empty() {
            return Err(ProviderError::Parse(
                "z.ai quota returned empty body (HTTP 200)".into(),
            ));
        }

        let (limits, _plan) = parse_quota_response(&body)?;
        snapshot_from_limits(&limits, updated_at, source)
    }
}
