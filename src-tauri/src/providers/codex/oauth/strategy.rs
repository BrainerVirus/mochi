use std::sync::Arc;

use async_trait::async_trait;

use super::client::{CodexOAuthClient, HttpCodexOAuthClient};
use super::credentials::current_timestamp;
use super::parse::snapshot_from_oauth_usage;
use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};

pub struct OAuthStrategy {
    client: Arc<dyn CodexOAuthClient>,
}

impl OAuthStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpCodexOAuthClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn CodexOAuthClient>) -> Self {
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
        "codex-oauth"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        match self.client.load_credentials().await {
            Ok(_) => Ok(true),
            Err(ProviderError::NotConfigured) => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let mut credentials = self.client.load_credentials().await?;

        if credentials.needs_refresh() && !credentials.refresh_token.is_empty() {
            credentials = self.client.refresh_credentials(&credentials).await?;
            self.client.save_credentials(&credentials).await?;
        }

        let usage = self
            .client
            .fetch_usage(&credentials.access_token, credentials.account_id.as_deref())
            .await?;

        snapshot_from_oauth_usage(&usage, &current_timestamp())
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(error, ProviderError::NotConfigured | ProviderError::Auth(_))
    }
}

#[cfg(test)]
mod tests {
    use super::super::credentials::CodexOAuthCredentials;
    use super::super::parse::CodexUsageResponse;
    use super::*;
    use time::OffsetDateTime;

    struct MockCodexOAuthClient {
        credentials: ProviderResult<CodexOAuthCredentials>,
        usage: ProviderResult<CodexUsageResponse>,
    }

    #[async_trait]
    impl CodexOAuthClient for MockCodexOAuthClient {
        async fn load_credentials(&self) -> ProviderResult<CodexOAuthCredentials> {
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
            _credentials: &CodexOAuthCredentials,
        ) -> ProviderResult<()> {
            Ok(())
        }

        async fn refresh_credentials(
            &self,
            credentials: &CodexOAuthCredentials,
        ) -> ProviderResult<CodexOAuthCredentials> {
            Ok(credentials.clone())
        }

        async fn fetch_usage(
            &self,
            _access_token: &str,
            _account_id: Option<&str>,
        ) -> ProviderResult<CodexUsageResponse> {
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

    fn fixture_credentials() -> CodexOAuthCredentials {
        CodexOAuthCredentials {
            access_token: "access-token".into(),
            refresh_token: "refresh-token".into(),
            id_token: None,
            account_id: Some("account-123".into()),
            last_refresh: Some(OffsetDateTime::now_utc()),
        }
    }

    fn fixture_usage_response() -> CodexUsageResponse {
        serde_json::from_str(include_str!("../fixtures/oauth_usage.json")).expect("fixture json")
    }

    #[tokio::test]
    async fn oauth_strategy_returns_snapshot_when_token_valid() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockCodexOAuthClient {
            credentials: Ok(fixture_credentials()),
            usage: Ok(fixture_usage_response()),
        }));

        let snapshot = strategy
            .fetch(&FetchContext)
            .await
            .expect("oauth fetch should succeed");

        assert_eq!(snapshot.source, "codex-oauth");
        assert_eq!(snapshot.primary.used_percent, 22.0);
        assert_eq!(snapshot.secondary.expect("secondary").used_percent, 43.0);
    }

    #[tokio::test]
    async fn oauth_strategy_unavailable_without_credentials() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockCodexOAuthClient {
            credentials: Err(ProviderError::NotConfigured),
            usage: Ok(fixture_usage_response()),
        }));

        let available = strategy
            .is_available(&FetchContext)
            .await
            .expect("availability check");

        assert!(!available);
    }

    #[tokio::test]
    async fn oauth_auth_errors_fallback_to_cli() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockCodexOAuthClient {
            credentials: Ok(fixture_credentials()),
            usage: Err(ProviderError::Auth("expired".into())),
        }));

        let error = strategy
            .fetch(&FetchContext)
            .await
            .expect_err("expired token should fail");

        assert!(strategy.should_fallback(&error));
    }
}
