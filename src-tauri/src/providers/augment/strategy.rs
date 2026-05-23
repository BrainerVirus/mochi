use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::timeout;

use super::client::{AugmentWebClient, HttpAugmentWebClient};
use super::credentials::{resolve_auggie_binary, resolve_session};
use super::usage_parse::{parse_auggie_cli_output, snapshot_from_parsed};
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct CliStrategy;

impl CliStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CliStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for CliStrategy {
    fn id(&self) -> &'static str {
        "augment-cli"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_auggie_binary().is_some())
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let binary = resolve_auggie_binary().ok_or(ProviderError::NotConfigured)?;
        let output = timeout(
            Duration::from_secs(15),
            Command::new(binary).args(["account", "status"]).output(),
        )
        .await
        .map_err(|_| ProviderError::Timeout)?
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Err(ProviderError::Fetch("Auggie CLI returned no output".into()));
        }

        let parsed = parse_auggie_cli_output(&stdout)?;
        Ok(snapshot_from_parsed(
            &parsed,
            &current_timestamp(),
            "augment-cli",
        ))
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::NotConfigured
                | ProviderError::Auth(_)
                | ProviderError::Timeout
                | ProviderError::Fetch(_)
                | ProviderError::Parse(_)
        )
    }
}

pub struct WebStrategy {
    client: Arc<dyn AugmentWebClient>,
}

impl WebStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpAugmentWebClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn AugmentWebClient>) -> Self {
        Self { client }
    }
}

impl Default for WebStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for WebStrategy {
    fn id(&self) -> &'static str {
        "augment-web"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_session(ctx.config(ProviderId::Augment))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let session = resolve_session(ctx.config(ProviderId::Augment))?
            .ok_or(ProviderError::NotConfigured)?;
        let parsed = self.client.fetch_usage(&session.cookie_header).await?;
        Ok(snapshot_from_parsed(
            &parsed,
            &current_timestamp(),
            "augment-web",
        ))
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::AugmentWebClient;
    use super::super::usage_parse::parse_web_responses;
    use super::*;
    use async_trait::async_trait;

    struct MockAugmentWebClient;

    #[async_trait]
    impl AugmentWebClient for MockAugmentWebClient {
        async fn fetch_usage(
            &self,
            _cookie_header: &str,
        ) -> ProviderResult<super::super::usage_parse::ParsedAugmentUsage> {
            parse_web_responses(
                include_str!("fixtures/credits.json"),
                Some(include_str!("fixtures/subscription.json")),
            )
        }
    }

    #[tokio::test]
    async fn web_strategy_returns_snapshot_with_cookie() {
        let strategy = WebStrategy::with_client(Arc::new(MockAugmentWebClient));
        let mut ctx = FetchContext::empty();
        ctx.provider_configs.insert(
            "augment".into(),
            crate::settings::ProviderConfig {
                manual_cookie: Some("session=test".into()),
                ..Default::default()
            },
        );

        let snapshot = strategy.fetch(&ctx).await.expect("fetch");
        assert_eq!(snapshot.source, "augment-web");
        assert_eq!(snapshot.primary.label, "Credits");
    }
}
