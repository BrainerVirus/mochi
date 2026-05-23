use async_trait::async_trait;
use reqwest::header::{ACCEPT, COOKIE};

use super::usage_parse::{parse_web_responses, ParsedAugmentUsage};
use crate::core::provider::{ProviderError, ProviderResult};

const BASE_URL: &str = "https://app.augmentcode.com";

#[async_trait]
pub trait AugmentWebClient: Send + Sync {
    async fn fetch_usage(&self, cookie_header: &str) -> ProviderResult<ParsedAugmentUsage>;
}

pub struct HttpAugmentWebClient {
    http: reqwest::Client,
}

impl HttpAugmentWebClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }
}

impl Default for HttpAugmentWebClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AugmentWebClient for HttpAugmentWebClient {
    async fn fetch_usage(&self, cookie_header: &str) -> ProviderResult<ParsedAugmentUsage> {
        let credits_url = format!("{BASE_URL}/api/credits");
        let credits_response = self
            .http
            .get(&credits_url)
            .header(ACCEPT, "application/json")
            .header(COOKIE, cookie_header)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let credits_status = credits_response.status();
        let credits_body = credits_response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        if credits_status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Auth("Augment session expired".into()));
        }
        if !credits_status.is_success() {
            return Err(ProviderError::Fetch(format!(
                "Augment credits API HTTP {}",
                credits_status
            )));
        }

        let subscription_body = match self
            .http
            .get(format!("{BASE_URL}/api/subscription"))
            .header(ACCEPT, "application/json")
            .header(COOKIE, cookie_header)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => response.text().await.ok(),
            _ => None,
        };

        parse_web_responses(&credits_body, subscription_body.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::super::usage_parse::parse_web_responses;
    use super::*;
    use async_trait::async_trait;

    struct MockAugmentWebClient;

    #[async_trait]
    impl AugmentWebClient for MockAugmentWebClient {
        async fn fetch_usage(&self, _cookie_header: &str) -> ProviderResult<ParsedAugmentUsage> {
            parse_web_responses(
                include_str!("fixtures/credits.json"),
                Some(include_str!("fixtures/subscription.json")),
            )
        }
    }

    #[tokio::test]
    async fn mock_client_returns_parsed_usage() {
        let client = MockAugmentWebClient;
        let parsed = client.fetch_usage("session=test").await.expect("fetch");
        assert_eq!(parsed.credits_used, 953170.0);
    }
}
