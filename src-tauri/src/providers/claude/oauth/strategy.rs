use std::sync::Arc;

use async_trait::async_trait;

use super::client::{ClaudeOAuthClient, HttpClaudeOAuthClient};
use super::credentials::current_timestamp;
use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::providers::claude::usage_parse::snapshot_from_usage_response;

pub struct OAuthStrategy {
    client: Arc<dyn ClaudeOAuthClient>,
}

impl OAuthStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpClaudeOAuthClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn ClaudeOAuthClient>) -> Self {
        Self { client }
    }
}

impl Default for OAuthStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for OAuthStrategy {
    fn id(&self) -> &'static str {
        "claude-oauth"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        match self.client.load_credentials().await {
            Ok(credentials) => Ok(credentials.has_user_profile_scope()),
            Err(ProviderError::NotConfigured) => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let mut credentials = self.client.load_credentials().await?;

        if !credentials.has_user_profile_scope() {
            return Err(ProviderError::Auth(
                "claude oauth token missing user:profile scope".into(),
            ));
        }

        if credentials.is_expired() {
            credentials = self.client.refresh_credentials(&credentials).await?;
            self.client.save_credentials(&credentials).await?;
        }

        let usage = self.client.fetch_usage(&credentials.access_token).await?;

        snapshot_from_usage_response(&usage, &current_timestamp(), "claude-oauth")
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(error, ProviderError::NotConfigured | ProviderError::Auth(_))
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::ClaudeOAuthClient;
    use super::super::credentials::parse_credentials;
    use super::*;
    use crate::providers::claude::usage_parse::ClaudeUsageResponse;

    struct MockClaudeOAuthClient {
        credentials: ProviderResult<super::super::credentials::ClaudeOAuthCredentials>,
        usage: ProviderResult<ClaudeUsageResponse>,
    }

    #[async_trait]
    impl ClaudeOAuthClient for MockClaudeOAuthClient {
        async fn load_credentials(
            &self,
        ) -> ProviderResult<super::super::credentials::ClaudeOAuthCredentials> {
            match &self.credentials {
                Ok(credentials) => Ok(credentials.clone()),
                Err(ProviderError::NotConfigured) => Err(ProviderError::NotConfigured),
                Err(ProviderError::Auth(message)) => Err(ProviderError::Auth(message.clone())),
                Err(ProviderError::Timeout) => Err(ProviderError::Timeout),
                Err(ProviderError::Parse(message)) => Err(ProviderError::Parse(message.clone())),
                Err(ProviderError::Fetch(message)) => Err(ProviderError::Fetch(message.clone())),
            }
        }

        async fn save_credentials(
            &self,
            _credentials: &super::super::credentials::ClaudeOAuthCredentials,
        ) -> ProviderResult<()> {
            Ok(())
        }

        async fn refresh_credentials(
            &self,
            credentials: &super::super::credentials::ClaudeOAuthCredentials,
        ) -> ProviderResult<super::super::credentials::ClaudeOAuthCredentials> {
            Ok(credentials.clone())
        }

        async fn fetch_usage(&self, _access_token: &str) -> ProviderResult<ClaudeUsageResponse> {
            match &self.usage {
                Ok(response) => Ok(response.clone()),
                Err(ProviderError::NotConfigured) => Err(ProviderError::NotConfigured),
                Err(ProviderError::Auth(message)) => Err(ProviderError::Auth(message.clone())),
                Err(ProviderError::Timeout) => Err(ProviderError::Timeout),
                Err(ProviderError::Parse(message)) => Err(ProviderError::Parse(message.clone())),
                Err(ProviderError::Fetch(message)) => Err(ProviderError::Fetch(message.clone())),
            }
        }
    }

    fn fixture_usage() -> ClaudeUsageResponse {
        serde_json::from_str(include_str!("../fixtures/oauth_usage.json")).expect("usage")
    }

    #[tokio::test]
    async fn oauth_strategy_returns_snapshot_when_token_valid() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockClaudeOAuthClient {
            credentials: Ok(
                parse_credentials(include_str!("../fixtures/credentials.json"))
                    .expect("credentials"),
            ),
            usage: Ok(fixture_usage()),
        }));

        let snapshot = strategy
            .fetch(&FetchContext)
            .await
            .expect("oauth fetch should succeed");

        assert_eq!(snapshot.source, "claude-oauth");
        assert_eq!(snapshot.primary.used_percent, 12.5);
    }

    #[tokio::test]
    async fn oauth_strategy_unavailable_without_credentials() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockClaudeOAuthClient {
            credentials: Err(ProviderError::NotConfigured),
            usage: Ok(fixture_usage()),
        }));

        let available = strategy.is_available(&FetchContext).await.expect("check");
        assert!(!available);
    }

    #[tokio::test]
    async fn oauth_auth_errors_fallback_to_web() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockClaudeOAuthClient {
            credentials: Ok(
                parse_credentials(include_str!("../fixtures/credentials.json"))
                    .expect("credentials"),
            ),
            usage: Err(ProviderError::Auth("expired".into())),
        }));

        let error = strategy.fetch(&FetchContext).await.expect_err("auth fail");
        assert!(strategy.should_fallback(&error));
    }
}
