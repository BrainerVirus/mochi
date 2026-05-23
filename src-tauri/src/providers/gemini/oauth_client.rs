//! Gemini CLI OAuth client ID/secret extraction.
//!
//! Derived from CodexBar `GeminiStatusProbe.swift` (MIT).

use std::path::{Path, PathBuf};

use crate::core::provider::{ProviderError, ProviderResult};

const ENV_CLI_PATH: &str = "MOCHI_GEMINI_CLI_PATH";
const ENV_CLIENT_ID: &str = "MOCHI_GEMINI_OAUTH_CLIENT_ID";
const ENV_CLIENT_SECRET: &str = "MOCHI_GEMINI_OAUTH_CLIENT_SECRET";
const OAUTH_FILE: &str = "dist/src/code_assist/oauth2.js";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthClientCredentials {
    pub client_id: String,
    pub client_secret: String,
}

pub fn resolve_oauth_client_credentials() -> ProviderResult<OAuthClientCredentials> {
    if let (Ok(client_id), Ok(client_secret)) = (
        std::env::var(ENV_CLIENT_ID),
        std::env::var(ENV_CLIENT_SECRET),
    ) {
        let client_id = client_id.trim();
        let client_secret = client_secret.trim();
        if !client_id.is_empty() && !client_secret.is_empty() {
            return Ok(OAuthClientCredentials {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
            });
        }
    }

    let gemini_path = resolve_gemini_binary()
        .ok_or_else(|| ProviderError::Auth("gemini cli not installed or not on PATH".into()))?;

    extract_oauth_credentials_from_binary(&gemini_path)
        .ok_or_else(|| ProviderError::Auth("could not find Gemini CLI OAuth configuration".into()))
}

pub fn parse_oauth_credentials(content: &str) -> Option<OAuthClientCredentials> {
    let client_id = extract_credential(content, "OAUTH_CLIENT_ID")?;
    let client_secret = extract_credential(content, "OAUTH_CLIENT_SECRET")?;
    Some(OAuthClientCredentials {
        client_id,
        client_secret,
    })
}

fn extract_credential(content: &str, name: &str) -> Option<String> {
    for quote in ['\'', '"'] {
        let marker = format!("{name} = {quote}");
        if let Some(start) = content.find(&marker) {
            let rest = &content[start + marker.len()..];
            if let Some(end) = rest.find(quote) {
                let value = rest[..end].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn extract_oauth_credentials_from_binary(gemini_path: &Path) -> Option<OAuthClientCredentials> {
    let resolved = resolve_symlinks(gemini_path);
    if let Some(credentials) = extract_from_legacy_paths(&resolved) {
        return Some(credentials);
    }

    find_gemini_package_root(&resolved)
        .and_then(|package_root| extract_oauth_credentials_from_package_root(&package_root))
}

fn extract_from_legacy_paths(gemini_path: &Path) -> Option<OAuthClientCredentials> {
    let bin_dir = gemini_path.parent()?;
    let base_dir = bin_dir.parent()?;

    let oauth_subpath = format!(
        "node_modules/@google/gemini-cli/node_modules/@google/gemini-cli-core/{OAUTH_FILE}"
    );
    let nix_share_subpath =
        format!("share/gemini-cli/node_modules/@google/gemini-cli-core/{OAUTH_FILE}");

    let candidates = [
        base_dir.join("libexec/lib").join(&oauth_subpath),
        base_dir.join("lib").join(&oauth_subpath),
        base_dir.join(&nix_share_subpath),
        base_dir.join("../gemini-cli-core").join(OAUTH_FILE),
        base_dir
            .join("node_modules/@google/gemini-cli-core")
            .join(OAUTH_FILE),
    ];

    candidates
        .iter()
        .find_map(|candidate| read_oauth_credentials_file(candidate))
}

fn extract_oauth_credentials_from_package_root(
    package_root: &Path,
) -> Option<OAuthClientCredentials> {
    [
        package_root.join(OAUTH_FILE),
        package_root
            .join("node_modules/@google/gemini-cli-core")
            .join(OAUTH_FILE),
    ]
    .iter()
    .find_map(|candidate| read_oauth_credentials_file(candidate))
}

fn read_oauth_credentials_file(path: &Path) -> Option<OAuthClientCredentials> {
    let content = std::fs::read_to_string(path).ok()?;
    parse_oauth_credentials(&content)
}

fn find_gemini_package_root(starting_at: &Path) -> Option<PathBuf> {
    let mut current = if starting_at.is_dir() {
        starting_at.to_path_buf()
    } else {
        starting_at.parent()?.to_path_buf()
    };

    for _ in 0..=8 {
        if is_gemini_package_root(&current) {
            return Some(current);
        }

        for layout in [
            current
                .join("lib")
                .join("node_modules")
                .join("@google")
                .join("gemini-cli"),
            current
                .join("libexec")
                .join("lib")
                .join("node_modules")
                .join("@google")
                .join("gemini-cli"),
        ] {
            if is_gemini_package_root(&layout) {
                return Some(layout);
            }
        }

        let parent = current.parent()?.to_path_buf();
        if parent == current {
            break;
        }
        current = parent;
    }

    None
}

fn is_gemini_package_root(path: &Path) -> bool {
    let package_json = path.join("package.json");
    let Ok(content) = std::fs::read_to_string(package_json) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    json.get("name").and_then(|value| value.as_str()) == Some("@google/gemini-cli")
}

fn resolve_gemini_binary() -> Option<PathBuf> {
    for key in [ENV_CLI_PATH, "GEMINI_CLI_PATH"] {
        if let Ok(path) = std::env::var(key) {
            let candidate = PathBuf::from(path.trim());
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    which_binary("gemini")
}

fn which_binary(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    let command = "where";
    #[cfg(not(windows))]
    let command = "which";

    let output = std::process::Command::new(command)
        .arg(name)
        .output()
        .ok()
        .filter(|result| result.status.success())?;

    let stdout = String::from_utf8(output.stdout).ok()?;
    let path = stdout.lines().next()?.trim();
    let candidate = PathBuf::from(path);
    candidate.is_file().then_some(candidate)
}

fn resolve_symlinks(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }

    #[test]
    fn parses_oauth_credentials_from_js_content() {
        let content = r#"
            const OAUTH_CLIENT_ID = 'test-client-id.apps.googleusercontent.com';
            const OAUTH_CLIENT_SECRET = 'test-client-secret';
        "#;
        let credentials = parse_oauth_credentials(content).expect("credentials");
        assert_eq!(
            credentials.client_id,
            "test-client-id.apps.googleusercontent.com"
        );
        assert_eq!(credentials.client_secret, "test-client-secret");
    }

    #[test]
    fn prefers_env_override_for_oauth_client_credentials() {
        let _guard = env_lock();
        std::env::set_var(ENV_CLIENT_ID, "env-client-id");
        std::env::set_var(ENV_CLIENT_SECRET, "env-client-secret");

        let credentials = resolve_oauth_client_credentials().expect("credentials");
        assert_eq!(credentials.client_id, "env-client-id");
        assert_eq!(credentials.client_secret, "env-client-secret");

        std::env::remove_var(ENV_CLIENT_ID);
        std::env::remove_var(ENV_CLIENT_SECRET);
    }
}
