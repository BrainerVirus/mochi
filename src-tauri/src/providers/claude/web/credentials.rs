//! Manual Claude web session credentials (`sessionKey` cookie).
//!
//! Derived from CodexBar `ClaudeWebAPIFetcher` manual cookie path (MIT).

use std::path::Path;

use crate::core::provider::{ProviderError, ProviderResult};

pub const ENV_SESSION_KEY: &str = "MOCHI_CLAUDE_SESSION_KEY";
pub const ENV_COOKIE: &str = "MOCHI_CLAUDE_COOKIE";
pub const ENV_COOKIE_FILE: &str = "MOCHI_CLAUDE_COOKIE_FILE";

pub fn resolve_session_key() -> ProviderResult<Option<String>> {
    if let Ok(value) = std::env::var(ENV_SESSION_KEY) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed.to_string()));
        }
    }

    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let session = session_key_from_cookie_header(&value)?;
        if session.is_some() {
            return Ok(session);
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
        .map_err(|error| ProviderError::Fetch(format!("read claude cookie file: {error}")))?;

    session_key_from_cookie_header(&data)
}

pub fn session_key_from_cookie_header(raw: &str) -> ProviderResult<Option<String>> {
    let trimmed = raw.trim().trim_start_matches("Cookie:").trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    for part in trimmed.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("sessionKey=") {
            let key = value.trim();
            if key.starts_with("sk-ant-") {
                return Ok(Some(key.to_string()));
            }
            return Err(ProviderError::Auth(
                "claude session key must start with sk-ant-".into(),
            ));
        }
    }

    if trimmed.starts_with("sk-ant-") {
        return Ok(Some(trimmed.to_string()));
    }

    Ok(None)
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
        let path = std::env::temp_dir().join(format!("mochi-claude-cookie-{nanos}.txt"));
        fs::write(&path, contents).expect("write");
        path
    }

    #[test]
    fn extracts_session_key_from_cookie_header() {
        let key = session_key_from_cookie_header("sessionKey=sk-ant-session123; path=/")
            .expect("parse")
            .expect("key");
        assert_eq!(key, "sk-ant-session123");
    }

    #[test]
    fn read_cookie_file_returns_session_key() {
        let path = temp_cookie_file("sessionKey=sk-ant-abc");
        let key = read_cookie_file(&path).expect("read").expect("key");
        assert_eq!(key, "sk-ant-abc");
        let _ = fs::remove_file(path);
    }
}
