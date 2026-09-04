use std::path::Path;

use crate::core::models::ProviderId;
use crate::settings::{
    load_settings, persist_settings, settings_file_path, MochiSettings, UpdateChannel,
};

/// Fields whose values must never reach stdout; `get` reports `<set>`/`<unset>`.
const SECRET_FIELDS: [&str; 4] = ["api_key", "admin_api_key", "manual_cookie", "token_account"];

fn settings_in(dir: &Path) -> MochiSettings {
    load_settings(&settings_file_path(dir))
}

fn channel_as_str(settings: &MochiSettings) -> String {
    serde_json::to_value(settings.update_channel)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "stable".to_string())
}

/// `update_channel` validated through the real settings deserialization, so an
/// invalid channel fails with the serde error instead of a hand-rolled copy.
fn parse_update_channel(value: &str) -> anyhow::Result<UpdateChannel> {
    let probe: MochiSettings = serde_json::from_value(serde_json::json!({
        "update_channel": value.trim(),
        "refresh_interval_seconds": 300,
        "enabled_providers": [],
        "show_notifications": true,
    }))
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(probe.update_channel)
}

fn parse_enabled_providers(value: &str) -> anyhow::Result<Vec<String>> {
    let mut providers = Vec::new();
    for raw in value.split(',') {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let provider =
            ProviderId::parse(raw).ok_or_else(|| anyhow::anyhow!("unknown provider: {raw}"))?;
        let canonical = provider.as_str().to_string();
        if !providers.contains(&canonical) {
            providers.push(canonical);
        }
    }
    Ok(providers)
}

fn split_provider_key(key: &str) -> Option<(ProviderId, &str)> {
    let (raw_provider, field) = key.split_once('.')?;
    if field.is_empty() || field.contains('.') {
        return None;
    }
    ProviderId::parse(raw_provider).map(|provider| (provider, field))
}

fn secret_value<'a>(
    settings: &'a MochiSettings,
    provider: ProviderId,
    field: &str,
) -> Option<&'a str> {
    let config = settings.provider_config(provider)?;
    match field {
        "api_key" => config.api_key_value(),
        "admin_api_key" => config.admin_api_key_value(),
        "manual_cookie" => config.manual_cookie_value(),
        "token_account" => config.token_account_value(),
        _ => None,
    }
}

fn cookie_source_of(settings: &MochiSettings, provider: ProviderId) -> String {
    settings
        .provider_config(provider)
        .and_then(|config| config.cookie_source.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "(unset)".to_string())
}

pub fn run_config_list(dir: &Path) -> anyhow::Result<String> {
    let settings = settings_in(dir);
    let mut lines = vec![
        format!("update_channel = {}", channel_as_str(&settings)),
        format!(
            "enabled_providers = {}",
            settings.enabled_providers.join(",")
        ),
    ];
    // Per-provider cookie sources derived from the registry; secret names and
    // values never appear here.
    for provider in ProviderId::all() {
        let source = cookie_source_of(&settings, *provider);
        if source != "(unset)" {
            lines.push(format!("{}.cookie_source = {source}", provider.as_str()));
        }
    }
    Ok(lines.join("\n"))
}

pub fn run_config_get(dir: &Path, key: &str) -> anyhow::Result<String> {
    let settings = settings_in(dir);
    let key = key.trim();
    match key {
        "update_channel" => Ok(channel_as_str(&settings)),
        "enabled_providers" => Ok(settings.enabled_providers.join(",")),
        _ => {
            let Some((provider, field)) = split_provider_key(key) else {
                return Err(anyhow::anyhow!("unknown key: {key}"));
            };
            if field == "cookie_source" {
                Ok(cookie_source_of(&settings, provider))
            } else if SECRET_FIELDS.contains(&field) {
                Ok(secret_value(&settings, provider, field)
                    .map(|_| "<set>")
                    .unwrap_or("<unset>")
                    .to_string())
            } else {
                Err(anyhow::anyhow!("unknown key: {key}"))
            }
        }
    }
}

pub fn run_config_set(dir: &Path, key: &str, value: &str) -> anyhow::Result<String> {
    let key = key.trim();
    let path = settings_file_path(dir);
    let mut settings = load_settings(&path);
    let display = match key {
        "update_channel" => {
            settings.update_channel = parse_update_channel(value)?;
            format!("update_channel = {}", channel_as_str(&settings))
        }
        "enabled_providers" => {
            settings.enabled_providers = parse_enabled_providers(value)?;
            format!(
                "enabled_providers = {}",
                settings.enabled_providers.join(",")
            )
        }
        _ => {
            let Some((provider, field)) = split_provider_key(key) else {
                return Err(anyhow::anyhow!("unknown key: {key}"));
            };
            if field == "cookie_source" {
                let trimmed = value.trim();
                let entry = settings
                    .provider_configs
                    .entry(provider.as_str().to_string())
                    .or_default();
                entry.cookie_source = (!trimmed.is_empty()).then(|| trimmed.to_string());
                settings.normalize_provider_ids();
                format!("{}.cookie_source = {trimmed}", provider.as_str())
            } else if SECRET_FIELDS.contains(&field) {
                return Err(anyhow::anyhow!("refusing to write secret '{key}' via CLI"));
            } else {
                return Err(anyhow::anyhow!("unknown key: {key}"));
            }
        }
    };
    persist_settings(&path, &settings).map_err(|error| anyhow::anyhow!(error))?;
    Ok(display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::ProviderConfig;

    fn test_dir(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "mochi-config-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).expect("test dir");
        dir
    }

    #[test]
    fn config_get_unknown_key_errors() {
        let dir = test_dir("unknown-key");
        let err = run_config_get(&dir, "nope").expect_err("unknown key");
        assert!(err.to_string().contains("unknown key"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_set_round_trips() {
        let dir = test_dir("round-trip");
        run_config_set(&dir, "update_channel", "stable").expect("set");
        assert_eq!(
            run_config_get(&dir, "update_channel").expect("get"),
            "stable"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_never_prints_secrets() {
        let dir = test_dir("never-prints");
        run_config_set(&dir, "update_channel", "stable").expect("set");
        let list = run_config_list(&dir).expect("list");
        assert!(!list.contains("session_token"));
        assert!(!list.contains("api_key"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_get_masks_secret_values() {
        let dir = test_dir("masks-secrets");
        let path = settings_file_path(&dir);
        let mut settings = load_settings(&path);
        settings.provider_configs.insert(
            "cursor".to_string(),
            ProviderConfig {
                api_key: Some("sk-live-secret".to_string()),
                ..ProviderConfig::default()
            },
        );
        persist_settings(&path, &settings).expect("seed");
        assert_eq!(
            run_config_get(&dir, "cursor.api_key").expect("get"),
            "<set>"
        );
        assert_eq!(
            run_config_get(&dir, "cursor.admin_api_key").expect("get"),
            "<unset>"
        );
        let list = run_config_list(&dir).expect("list");
        assert!(!list.contains("sk-live-secret"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
