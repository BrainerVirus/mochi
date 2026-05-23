//! z.ai API token resolution.
//!
//! Derived from CodexBar `ZaiSettingsReader.swift` (MIT).

use std::path::Path;

use crate::core::provider::{ProviderError, ProviderResult};
use crate::settings::ProviderConfig;

const ENV_API_KEY: &str = "Z_AI_API_KEY";
const ENV_API_HOST: &str = "Z_AI_API_HOST";
const ENV_QUOTA_URL: &str = "Z_AI_QUOTA_URL";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZaiRegion {
    #[default]
    Global,
    BigModelCn,
}

pub fn resolve_api_key(config: Option<&ProviderConfig>) -> ProviderResult<Option<String>> {
    if let Some(key) = config.and_then(ProviderConfig::api_key_value) {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed.to_string()));
        }
    }

    if let Ok(value) = std::env::var(ENV_API_KEY) {
        let trimmed = value.trim().trim_matches('"');
        if !trimmed.is_empty() {
            return Ok(Some(trimmed.to_string()));
        }
    }

    Ok(None)
}

pub fn resolve_region(config: Option<&ProviderConfig>) -> ZaiRegion {
    let host = config
        .and_then(|cfg| cfg.region_host.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| std::env::var(ENV_API_HOST).ok());

    match host.as_deref().map(str::trim) {
        Some(value) if value.contains("bigmodel") => ZaiRegion::BigModelCn,
        _ => ZaiRegion::Global,
    }
}

pub fn quota_url(region: ZaiRegion) -> String {
    if let Ok(override_url) = std::env::var(ENV_QUOTA_URL) {
        let trimmed = override_url.trim();
        if !trimmed.is_empty() {
            return normalize_quota_url(trimmed);
        }
    }

    if let Ok(host) = std::env::var(ENV_API_HOST) {
        let trimmed = host.trim();
        if !trimmed.is_empty() {
            return normalize_quota_url(trimmed);
        }
    }

    match region {
        ZaiRegion::Global => "https://api.z.ai/api/monitor/usage/quota/limit".into(),
        ZaiRegion::BigModelCn => {
            "https://open.bigmodel.cn/api/monitor/usage/quota/limit".into()
        }
    }
}

fn normalize_quota_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.contains("/api/monitor/usage/quota/limit") {
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return trimmed.to_string();
        }
        return format!("https://{trimmed}");
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return format!(
            "{}/api/monitor/usage/quota/limit",
            trimmed.trim_end_matches('/')
        );
    }
    format!("https://{trimmed}/api/monitor/usage/quota/limit")
}

#[allow(dead_code)]
pub fn read_token_file(path: &Path) -> ProviderResult<Option<String>> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Auth(format!("read token file: {error}")))?;
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}
