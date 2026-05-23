use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::settings::{MochiSettings, ProviderConfig};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub id: ProviderId,
    pub display_name: String,
    pub supports_status: bool,
    pub supports_cost: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FetchKind {
    Cli,
    OAuth,
    ApiKey,
    BrowserCookies,
    LocalConfig,
    LocalProbe,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider is not configured")]
    NotConfigured,
    #[error("provider fetch timed out")]
    Timeout,
    #[error("provider auth failed: {0}")]
    Auth(String),
    #[error("provider parse failed: {0}")]
    Parse(String),
    #[error("provider fetch failed: {0}")]
    Fetch(String),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[derive(Debug, Clone, Default)]
pub struct FetchContext {
    pub provider_configs: HashMap<String, ProviderConfig>,
}

impl FetchContext {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_settings(settings: &MochiSettings) -> Self {
        Self {
            provider_configs: settings.provider_configs.clone(),
        }
    }

    pub fn config(&self, provider: ProviderId) -> Option<&ProviderConfig> {
        self.provider_configs.get(provider.as_str())
    }
}

#[async_trait]
pub trait FetchStrategy: Send + Sync {
    fn id(&self) -> &'static str;
    fn kind(&self) -> FetchKind;
    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool>;
    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot>;
    fn should_fallback(&self, error: &ProviderError) -> bool;
}

pub trait Provider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn strategies(&self) -> Vec<Box<dyn FetchStrategy>>;
}

#[async_trait]
pub trait ProviderEnrichment: Send + Sync {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}
