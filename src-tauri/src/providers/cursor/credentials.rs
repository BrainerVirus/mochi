//! Cursor session cookie resolution for web API fetch.
//!
//! Derived from CodexBar `docs/cursor.md` (MIT). Browser cookie import is deferred;
//! cookies are supplied via env or file until cross-platform import lands.

use std::path::Path;

use crate::core::provider::{ProviderError, ProviderResult};
use crate::settings::ProviderConfig;

const ENV_COOKIE: &str = "MOCHI_CURSOR_COOKIE";
const ENV_COOKIE_FILE: &str = "MOCHI_CURSOR_COOKIE_FILE";

pub fn resolve_manual_cookie(config: Option<&ProviderConfig>) -> ProviderResult<Option<String>> {
    if let Some(cookie) = config.and_then(ProviderConfig::manual_cookie_value) {
        return Ok(Some(normalize_cookie_header(cookie)));
    }

    resolve_manual_cookie_from_env()
}

pub fn resolve_manual_cookie_from_env() -> ProviderResult<Option<String>> {
    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(normalize_cookie_header(trimmed)));
        }
    }

    if let Ok(path) = std::env::var(ENV_COOKIE_FILE) {
        return read_cookie_file(Path::new(&path));
    }

    Ok(None)
}

fn read_cookie_file(path: &Path) -> ProviderResult<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }

    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read cursor cookie file: {error}")))?;
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(normalize_cookie_header(trimmed)))
}

pub fn normalize_cookie_header(raw: &str) -> String {
    raw.trim().trim_start_matches("Cookie:").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_cookie_header_strips_prefix() {
        assert_eq!(
            normalize_cookie_header("Cookie: WorkosCursorSessionToken=abc"),
            "WorkosCursorSessionToken=abc"
        );
    }
}
