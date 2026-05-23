use std::path::Path;

use crate::core::provider::{ProviderError, ProviderResult};
use crate::settings::ProviderConfig;

const ENV_COOKIE: &str = "MOCHI_CODEX_COOKIE";
const ENV_COOKIE_FILE: &str = "MOCHI_CODEX_COOKIE_FILE";

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
        .map_err(|error| ProviderError::Fetch(format!("read codex cookie file: {error}")))?;
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(normalize_cookie_header(trimmed)))
}

fn normalize_cookie_header(raw: &str) -> String {
    raw.trim().trim_start_matches("Cookie:").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_cookie_file(contents: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mochi-codex-cookie-{nanos}.txt"));
        fs::write(&path, contents).expect("write cookie file");
        path
    }

    #[test]
    fn normalize_cookie_header_strips_prefix() {
        assert_eq!(
            normalize_cookie_header("Cookie: session=abc; path=/"),
            "session=abc; path=/"
        );
    }

    #[test]
    fn read_cookie_file_returns_trimmed_value() {
        let path = temp_cookie_file("  session=abc  \n");
        let cookie = read_cookie_file(&path).expect("read cookie");
        assert_eq!(cookie.as_deref(), Some("session=abc"));
        let _ = fs::remove_file(path);
    }
}
