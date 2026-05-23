//! OpenCode session cookie resolution.
//!
//! Derived from CodexBar `OpenCodeCookieImporter.swift` (MIT).

use crate::browser::opencode;
use crate::core::provider::ProviderResult;
use crate::settings::ProviderConfig;

const ENV_COOKIE: &str = "MOCHI_OPENCODE_COOKIE";
const ENV_WORKSPACE_ID: &str = "MOCHI_OPENCODE_WORKSPACE_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOpenCodeSession {
    pub cookie_header: String,
    pub source_label: String,
    pub workspace_id: Option<String>,
}

pub fn resolve_session(config: Option<&ProviderConfig>) -> ProviderResult<Option<ResolvedOpenCodeSession>> {
    if config.is_some_and(ProviderConfig::cookie_source_is_off) {
        return Ok(None);
    }

    let workspace_id = resolve_workspace_id(config);

    if let Some(manual) = config.and_then(ProviderConfig::manual_cookie_value) {
        return Ok(Some(ResolvedOpenCodeSession {
            cookie_header: normalize_cookie_header(manual),
            source_label: "Manual".into(),
            workspace_id,
        }));
    }

    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(ResolvedOpenCodeSession {
                cookie_header: normalize_cookie_header(trimmed),
                source_label: "Environment".into(),
                workspace_id,
            }));
        }
    }

    if config.is_none_or(|cfg| !cfg.cookie_source_is_manual()) {
        if let Some(home) = user_home_dir() {
            if let Some(imported) = opencode::import_from_browsers(&home) {
                return Ok(Some(ResolvedOpenCodeSession {
                    cookie_header: imported.cookie_header,
                    source_label: imported.source_label,
                    workspace_id,
                }));
            }
        }
    }

    Ok(None)
}

pub fn resolve_workspace_id(config: Option<&ProviderConfig>) -> Option<String> {
    if let Ok(value) = std::env::var(ENV_WORKSPACE_ID) {
        if let Some(normalized) = normalize_workspace_id(&value) {
            return Some(normalized);
        }
    }

    config
        .and_then(|cfg| cfg.token_account.as_deref())
        .and_then(normalize_workspace_id)
}

pub fn normalize_workspace_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.starts_with("wrk_") && trimmed.len() > 4 {
        return Some(trimmed.to_string());
    }

    if let Some(index) = trimmed.find("wrk_") {
        let candidate = &trimmed[index..];
        if let Some(end) = candidate.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) {
            return Some(candidate[..end].to_string());
        }
        return Some(candidate.to_string());
    }

    None
}

fn normalize_cookie_header(raw: &str) -> String {
    raw.trim()
        .trim_start_matches("Cookie:")
        .trim()
        .to_string()
}

fn user_home_dir() -> Option<std::path::PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return Some(std::path::PathBuf::from(home));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            if !home.trim().is_empty() {
                return Some(std::path::PathBuf::from(home));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_workspace_id_from_url() {
        assert_eq!(
            normalize_workspace_id("https://opencode.ai/workspace/wrk_abc123/billing"),
            Some("wrk_abc123".into())
        );
    }
}
