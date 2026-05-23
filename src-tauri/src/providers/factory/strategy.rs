use std::sync::Arc;

use async_trait::async_trait;

use super::client::{FactoryWebClient, HttpFactoryWebClient};
use super::credentials::resolve_session;
use super::usage_parse::snapshot_from_parsed;
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct WebCookiesStrategy {
    client: Arc<dyn FactoryWebClient>,
}

impl WebCookiesStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpFactoryWebClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn FactoryWebClient>) -> Self {
        Self { client }
    }
}

impl Default for WebCookiesStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for WebCookiesStrategy {
    fn id(&self) -> &'static str {
        "factory-web-cookies"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_session(ctx.config(ProviderId::Factory))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let session = resolve_session(ctx.config(ProviderId::Factory))?
            .ok_or(ProviderError::NotConfigured)?;
        let parsed = self
            .client
            .fetch_usage(&session.cookie_header, session.bearer_token.as_deref())
            .await?;
        Ok(snapshot_from_parsed(
            &parsed,
            &current_timestamp(),
            "factory-web-cookies",
        ))
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::FactoryWebClient;
    use super::super::usage_parse::{parse_auth_me, parse_usage_response};
    use super::*;
    use async_trait::async_trait;

    struct MockFactoryWebClient;

    #[async_trait]
    impl FactoryWebClient for MockFactoryWebClient {
        async fn fetch_usage(
            &self,
            _cookie_header: &str,
            _bearer_token: Option<&str>,
        ) -> ProviderResult<super::super::usage_parse::ParsedFactoryUsage> {
            let (_, organization, tier, plan) =
                parse_auth_me(include_str!("fixtures/auth_me.json"))?;
            let mut parsed = parse_usage_response(include_str!("fixtures/usage.json"))?;
            parsed.organization_name = organization;
            parsed.tier = tier;
            parsed.plan_name = plan;
            Ok(parsed)
        }
    }

    #[tokio::test]
    async fn web_strategy_returns_standard_and_premium_windows() {
        let strategy = WebCookiesStrategy::with_client(Arc::new(MockFactoryWebClient));
        let mut ctx = FetchContext::empty();
        ctx.provider_configs.insert(
            "factory".into(),
            crate::settings::ProviderConfig {
                manual_cookie: Some("session=factory".into()),
                ..Default::default()
            },
        );

        let snapshot = strategy.fetch(&ctx).await.expect("fetch");
        assert_eq!(snapshot.source, "factory-web-cookies");
        assert_eq!(snapshot.primary.label, "Standard");
        assert_eq!(
            snapshot.secondary.as_ref().expect("premium").label,
            "Premium"
        );
    }
}
