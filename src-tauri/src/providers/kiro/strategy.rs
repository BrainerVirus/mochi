use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::timeout;

use super::usage_parse::{parse_usage_output, snapshot_from_parsed};
use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

const USAGE_TIMEOUT: Duration = Duration::from_secs(20);

pub struct CliUsageStrategy;

pub(crate) fn resolve_kiro_binary() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(if cfg!(windows) {
                "kiro-cli.exe"
            } else {
                "kiro-cli"
            });
            candidate.is_file().then_some(candidate)
        })
    })
}

impl CliUsageStrategy {
    pub fn new() -> Self {
        Self
    }

    async fn run_usage_command() -> ProviderResult<String> {
        let binary = resolve_kiro_binary().ok_or(ProviderError::NotConfigured)?;
        let output = timeout(
            USAGE_TIMEOUT,
            Command::new(binary)
                .args(["chat", "--no-interactive", "/usage"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .env("TERM", "xterm-256color")
                .output(),
        )
        .await
        .map_err(|_| ProviderError::Timeout)?
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        if !stdout.trim().is_empty() {
            return Ok(stdout);
        }
        if !stderr.trim().is_empty() {
            return Ok(stderr);
        }

        Err(ProviderError::Fetch("kiro-cli returned no output".into()))
    }
}

impl Default for CliUsageStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for CliUsageStrategy {
    fn id(&self) -> &'static str {
        "kiro-cli-usage"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_kiro_binary().is_some())
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let output = Self::run_usage_command().await?;
        let parsed = parse_usage_output(&output)?;
        Ok(snapshot_from_parsed(
            &parsed,
            &current_timestamp(),
            "kiro-cli-usage",
        ))
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}
