use std::path::PathBuf;

use async_trait::async_trait;

use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};

pub struct BrowserCookiesStrategy {
    auth_path: PathBuf,
}

impl BrowserCookiesStrategy {
    pub fn new() -> Self {
        Self::with_auth_path(default_codex_auth_path())
    }

    pub fn with_auth_path(auth_path: PathBuf) -> Self {
        Self { auth_path }
    }
}

impl Default for BrowserCookiesStrategy {
    fn default() -> Self {
        Self::new()
    }
}

fn default_codex_auth_path() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .map(|home| home.join(".codex").join("auth.json"))
        .unwrap_or_else(|| PathBuf::from(".codex/auth.json"))
}

#[async_trait]
impl FetchStrategy for BrowserCookiesStrategy {
    fn id(&self) -> &'static str {
        "codex-browser-cookies"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(self.auth_path.is_file())
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        if !self.auth_path.is_file() {
            return Err(ProviderError::NotConfigured);
        }

        Err(ProviderError::Fetch(
            "codex browser cookie usage fetch is not implemented yet".into(),
        ))
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::NotConfigured
                | ProviderError::Auth(_)
                | ProviderError::Timeout
                | ProviderError::Fetch(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_auth_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("mochi-codex-auth-{nanos}.json"))
    }

    #[tokio::test]
    async fn is_available_when_auth_file_exists() {
        let auth_path = temp_auth_path();
        fs::write(&auth_path, "{}").expect("write auth fixture");
        let strategy = BrowserCookiesStrategy::with_auth_path(auth_path.clone());

        assert!(strategy
            .is_available(&FetchContext)
            .await
            .expect("availability"));

        let _ = fs::remove_file(auth_path);
    }

    #[tokio::test]
    async fn fetch_reports_not_configured_without_auth_file() {
        let strategy =
            BrowserCookiesStrategy::with_auth_path(PathBuf::from("/tmp/mochi-missing-auth.json"));

        let error = strategy
            .fetch(&FetchContext)
            .await
            .expect_err("missing auth should fail");

        assert!(matches!(error, ProviderError::NotConfigured));
    }

    #[tokio::test]
    async fn unfinished_cookie_fetch_can_fallback_when_auth_file_exists() {
        let auth_path = temp_auth_path();
        fs::write(&auth_path, "{}").expect("write auth fixture");
        let strategy = BrowserCookiesStrategy::with_auth_path(auth_path.clone());

        let error = strategy
            .fetch(&FetchContext)
            .await
            .expect_err("unfinished cookie fetch should fail");

        assert!(strategy.should_fallback(&error));

        let _ = fs::remove_file(auth_path);
    }
}
