use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

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
    match settings.update_channel {
        UpdateChannel::Stable => "stable".to_string(),
    }
}

/// Minimal probe so `update_channel` is validated through the real
/// `UpdateChannel` deserialization without hardcoding unrelated settings fields.
#[derive(Debug, Deserialize)]
struct ChannelProbe {
    update_channel: UpdateChannel,
}

/// Minimal probe so `enabled_providers` entries are validated through the real
/// `ProviderId` deserialization (aliases, canonical spellings) instead of only
/// the hand-rolled `ProviderId::parse`.
#[derive(Debug, Deserialize)]
struct ProvidersProbe {
    enabled_providers: Vec<ProviderId>,
}

/// `update_channel` validated through the real settings deserialization, so an
/// invalid channel fails with the serde error instead of a hand-rolled copy.
fn parse_update_channel(value: &str) -> anyhow::Result<UpdateChannel> {
    let probe: ChannelProbe = serde_json::from_value(serde_json::json!({
        "update_channel": value.trim(),
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
    // Round-trip through real serde so the accepted spellings stay tied to
    // the `ProviderId` schema rather than drifting from it.
    let probe: ProvidersProbe = serde_json::from_value(serde_json::json!({
        "enabled_providers": providers,
    }))
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(probe
        .enabled_providers
        .iter()
        .map(|provider| provider.as_str().to_string())
        .collect())
}

/// Refuses to proceed when a settings file is present but unreadable, so a
/// `set` never silently discards user data by persisting defaults over it.
fn refuse_unreadable_settings(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let contents = std::fs::read_to_string(path).map_err(|error| {
        anyhow::anyhow!(
            "refusing to overwrite unreadable settings at {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str::<MochiSettings>(&contents).map_err(|error| {
        anyhow::anyhow!(
            "refusing to overwrite unreadable settings at {}: {error}",
            path.display()
        )
    })?;
    Ok(())
}

/// Splits `provider.field` keys. Field names are case-sensitive and matched
/// verbatim (`cookie_source` plus the secret names); only surrounding
/// whitespace around the field segment is trimmed.
fn split_provider_key(key: &str) -> Option<(ProviderId, &str)> {
    let (raw_provider, field) = key.split_once('.')?;
    let field = field.trim();
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

/// The same key/value pairs `run_config_list` prints, as a map so the
/// `--json` output and the human-readable output cannot drift apart.
/// Secret names and values never appear here.
pub fn config_list_map(dir: &Path) -> BTreeMap<String, String> {
    let settings = settings_in(dir);
    let mut map = BTreeMap::from([
        ("update_channel".to_string(), channel_as_str(&settings)),
        (
            "enabled_providers".to_string(),
            settings.enabled_providers.join(","),
        ),
    ]);
    // Per-provider cookie sources derived from the registry; secret names and
    // values never appear here.
    for provider in ProviderId::all() {
        let source = cookie_source_of(&settings, *provider);
        if source != "(unset)" {
            map.insert(format!("{}.cookie_source", provider.as_str()), source);
        }
    }
    map
}

pub fn run_config_list(dir: &Path) -> anyhow::Result<String> {
    Ok(config_list_map(dir)
        .iter()
        .map(|(key, value)| format!("{key} = {value}"))
        .collect::<Vec<_>>()
        .join("\n"))
}

/// `config --json`: the list map as JSON (still masked — secrets never enter
/// the map).
pub fn format_config_list_json(dir: &Path) -> anyhow::Result<String> {
    serde_json::to_string(&config_list_map(dir)).map_err(|error| anyhow::anyhow!(error.to_string()))
}

/// `config <key> [--json]` / `config <key> <value> [--json]`: single-pair JSON
/// matching the human-readable `key = value` display line.
pub fn format_config_value_json(key: &str, display: &str) -> anyhow::Result<String> {
    let value = display
        .split_once(" = ")
        .map(|(_, value)| value)
        .unwrap_or(display);
    serde_json::to_string(&BTreeMap::from([(
        key.trim().to_string(),
        value.to_string(),
    )]))
    .map_err(|error| anyhow::anyhow!(error.to_string()))
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
    refuse_unreadable_settings(&path)?;
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
                let shown = if trimmed.is_empty() {
                    "(unset)"
                } else {
                    trimmed
                };
                format!("{}.cookie_source = {shown}", provider.as_str())
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
    fn config_set_refuses_corrupt_settings_without_touching_file() {
        let dir = test_dir("corrupt-guard");
        let path = settings_file_path(&dir);
        std::fs::write(&path, "{ not valid json !!!").expect("seed garbage");
        let before = std::fs::read(&path).expect("read seed");

        let err = run_config_set(&dir, "update_channel", "stable").expect_err("corrupt file");
        assert!(
            err.to_string()
                .contains("refusing to overwrite unreadable settings"),
            "unexpected error: {err}"
        );
        assert_eq!(std::fs::read(&path).expect("read after"), before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_set_rejects_bogus_channel_and_leaves_file_intact() {
        let dir = test_dir("bogus-channel");
        run_config_set(&dir, "update_channel", "stable").expect("seed");
        let path = settings_file_path(&dir);
        let before = std::fs::read(&path).expect("read seed");

        let err = run_config_set(&dir, "update_channel", "bogus").expect_err("bogus channel");
        assert!(
            err.to_string().contains("unknown update channel"),
            "unexpected: {err}"
        );
        assert_eq!(std::fs::read(&path).expect("read after"), before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_set_rejects_unknown_provider_and_leaves_file_intact() {
        let dir = test_dir("unknown-provider");
        run_config_set(&dir, "update_channel", "stable").expect("seed");
        let path = settings_file_path(&dir);
        let before = std::fs::read(&path).expect("read seed");

        let err = run_config_set(&dir, "enabled_providers", "cursor,nope-provider")
            .expect_err("unknown provider");
        assert!(
            err.to_string().contains("unknown provider"),
            "unexpected: {err}"
        );
        assert_eq!(std::fs::read(&path).expect("read after"), before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_set_refuses_secret_api_key_and_leaves_file_intact() {
        let dir = test_dir("refuse-secret");
        run_config_set(&dir, "update_channel", "stable").expect("seed");
        let path = settings_file_path(&dir);
        let before = std::fs::read(&path).expect("read seed");

        let err = run_config_set(&dir, "cursor.api_key", "sk-live").expect_err("secret refused");
        assert!(
            err.to_string().contains("refusing to write secret"),
            "unexpected: {err}"
        );
        assert_eq!(std::fs::read(&path).expect("read after"), before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_set_leaves_complete_valid_file() {
        let dir = test_dir("valid-file");
        run_config_set(&dir, "enabled_providers", "cursor,claude").expect("set");
        let path = settings_file_path(&dir);

        let contents = std::fs::read_to_string(&path).expect("read file");
        let parsed: MochiSettings = serde_json::from_str(&contents).expect("valid settings");
        assert_eq!(
            parsed.enabled_providers,
            vec!["cursor".to_string(), "claude".to_string()]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_set_clear_cookie_source_prints_unset() {
        let dir = test_dir("clear-cookie");
        run_config_set(&dir, "cursor.cookie_source", "manual").expect("seed");
        let display = run_config_set(&dir, "cursor.cookie_source", "  ").expect("clear");
        assert_eq!(display, "cursor.cookie_source = (unset)");
        assert_eq!(
            run_config_get(&dir, "cursor.cookie_source").expect("get"),
            "(unset)"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_dot_notation_trims_field_segment() {
        let dir = test_dir("trim-field");
        run_config_set(&dir, "cursor.cookie_source", "manual").expect("seed");
        assert_eq!(
            run_config_get(&dir, "cursor. cookie_source ").expect("get"),
            "manual"
        );
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

    #[test]
    fn config_list_json_matches_text_and_masks_secrets() {
        let dir = test_dir("list-json");
        run_config_set(&dir, "cursor.cookie_source", "manual").expect("seed");
        let path = settings_file_path(&dir);
        let mut settings = load_settings(&path);
        settings.provider_configs.insert(
            "cursor".to_string(),
            ProviderConfig {
                api_key: Some("sk-live-secret".to_string()),
                cookie_source: Some("manual".to_string()),
                ..ProviderConfig::default()
            },
        );
        persist_settings(&path, &settings).expect("seed secret");

        let text = run_config_list(&dir).expect("list");
        let json = format_config_list_json(&dir).expect("json");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parses");
        assert!(!json.contains("sk-live-secret"), "leak: {json}");
        assert_eq!(parsed["cursor.cookie_source"], serde_json::json!("manual"));
        for line in text.lines() {
            let (key, _) = line.split_once(" = ").expect("line");
            assert!(parsed.get(key).is_some(), "missing {key} in {json}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
