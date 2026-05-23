//! Optional import of provider credentials from CodexBar config files.

use std::path::{Path, PathBuf};

use super::{ProviderConfig, TokenAccountData};

#[derive(Debug, serde::Deserialize)]
struct CodexBarConfig {
    #[serde(default)]
    providers: Vec<CodexBarProviderConfig>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexBarProviderConfig {
    id: String,
    #[serde(default, rename = "cookieHeader")]
    cookie_header: Option<String>,
    #[serde(default, rename = "cookieSource")]
    cookie_source: Option<String>,
    #[serde(default, rename = "workspaceID")]
    workspace_id: Option<String>,
    #[serde(default, rename = "tokenAccounts")]
    token_accounts: Option<TokenAccountData>,
}

pub fn codexbar_config_path(home: &Path) -> PathBuf {
    home.join(".codexbar").join("config.json")
}

pub fn load_provider_config(provider_id: &str) -> Option<ProviderConfig> {
    let home = user_home_dir()?;
    let path = codexbar_config_path(&home);
    let contents = std::fs::read_to_string(path).ok()?;
    let config: CodexBarConfig = serde_json::from_str(&contents).ok()?;
    config
        .providers
        .into_iter()
        .find(|entry| entry.id == provider_id)
        .map(|entry| ProviderConfig {
            cookie_source: entry.cookie_source,
            manual_cookie: entry.cookie_header,
            workspace_id: entry.workspace_id,
            token_accounts: entry.token_accounts,
            ..ProviderConfig::default()
        })
}

pub fn merge_codexbar_token_accounts(
    config: Option<&ProviderConfig>,
    provider_id: &str,
) -> Option<ProviderConfig> {
    if config.is_some_and(|cfg| {
        cfg.manual_cookie_value().is_some() || cfg.active_token_account().is_some()
    }) {
        return config.cloned();
    }

    load_provider_config(provider_id)
}

fn user_home_dir() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return Some(PathBuf::from(home));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            if !home.trim().is_empty() {
                return Some(PathBuf::from(home));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_codexbar_opencodego_provider_config() {
        let temp = std::env::temp_dir().join(format!(
            "mochi-codexbar-import-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(temp.join(".codexbar")).expect("dir");
        fs::write(
            temp.join(".codexbar/config.json"),
            r#"{
              "providers": [{
                "id": "opencodego",
                "cookieSource": "manual",
                "cookieHeader": "auth=legacy",
                "workspaceID": "wrk_test",
                "tokenAccounts": {
                  "version": 1,
                  "accounts": [{ "id": "acc-1", "label": "zen", "token": "oc_locale=en; auth=Fe26.test" }],
                  "activeIndex": 0
                }
              }]
            }"#,
        )
        .expect("write config");

        let path = codexbar_config_path(&temp);
        let contents = fs::read_to_string(path).expect("read");
        let config: CodexBarConfig = serde_json::from_str(&contents).expect("parse");
        let entry = config.providers.into_iter().next().expect("provider");

        assert_eq!(entry.id, "opencodego");
        assert_eq!(entry.workspace_id.as_deref(), Some("wrk_test"));
        let accounts = entry.token_accounts.expect("token accounts");
        assert_eq!(accounts.accounts.len(), 1);
        assert_eq!(accounts.accounts[0].label, "zen");

        let _ = fs::remove_dir_all(temp);
    }
}
