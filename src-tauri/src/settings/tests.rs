use super::*;

#[test]
fn defaults_to_stable_channel() {
    let settings = MochiSettings::default();
    assert_eq!(settings.update_channel, UpdateChannel::Stable);
    assert!(settings.enabled_providers.is_empty());
}

#[test]
fn update_channel_defaults_to_stable() {
    let settings: MochiSettings = serde_json::from_value(serde_json::json!({
        "refresh_interval_seconds": 300,
        "enabled_providers": [],
        "show_notifications": true,
    }))
    .expect("parse");
    assert_eq!(settings.update_channel, UpdateChannel::Stable);
}

#[test]
fn update_channel_round_trips_kebab_case() {
    let settings: MochiSettings = serde_json::from_value(serde_json::json!({
        "update_channel": "stable",
        "refresh_interval_seconds": 300,
        "enabled_providers": [],
        "show_notifications": true,
    }))
    .expect("lowercase channel should parse");

    let json = serde_json::to_string(&settings).expect("settings should serialize");
    assert!(json.contains("\"update_channel\":\"stable\""));
}

#[test]
fn legacy_unstable_channel_maps_to_stable_and_rest_survives() {
    let settings: MochiSettings = serde_json::from_value(serde_json::json!({
        "update_channel": "unstable",
        "refresh_interval_seconds": 120,
        "enabled_providers": ["opencode"],
        "show_notifications": false,
        "provider_configs": {
            "opencode": { "api_key": "sk-test" }
        },
    }))
    .expect("legacy channel should not discard the whole settings file");

    assert_eq!(settings.update_channel, UpdateChannel::Stable);
    assert_eq!(settings.refresh_interval_seconds, 120);
    assert_eq!(settings.enabled_providers, vec!["opencode".to_string()]);
    assert!(!settings.show_notifications);
    assert_eq!(
        settings
            .provider_config(ProviderId::OpenCode)
            .and_then(|config| config.api_key_value()),
        Some("sk-test")
    );
}

#[test]
fn normalize_provider_ids_maps_codexbar_aliases() {
    let mut settings = MochiSettings {
        enabled_providers: vec!["opencodego".into(), "open-code".into()],
        provider_configs: HashMap::from([(
            "opencodego".into(),
            ProviderConfig {
                manual_cookie: Some("auth=test".into()),
                ..ProviderConfig::default()
            },
        )]),
        ..MochiSettings::default()
    };

    settings.normalize_provider_ids();

    assert_eq!(
        settings.enabled_providers,
        vec!["opencode-go".to_string(), "opencode".to_string()]
    );
    assert!(settings.provider_config(ProviderId::OpenCodeGo).is_some());
}

#[test]
fn normalize_provider_ids_canonicalizes_selected_tab_alias() {
    let mut settings = MochiSettings {
        selected_tab: Some("opencodego".into()),
        ..MochiSettings::default()
    };

    settings.normalize_provider_ids();

    assert_eq!(settings.selected_tab.as_deref(), Some("opencode-go"));
}

#[test]
fn normalize_provider_ids_clears_invalid_selected_tab() {
    let mut settings = MochiSettings {
        selected_tab: Some("';globalThis.pwned=true;//".into()),
        ..MochiSettings::default()
    };

    settings.normalize_provider_ids();

    assert_eq!(settings.selected_tab, None);
}

#[test]
fn token_account_data_returns_active_account() {
    let data = TokenAccountData {
        version: 1,
        accounts: vec![TokenAccount {
            id: "a".into(),
            label: "zen".into(),
            token: "auth=test".into(),
        }],
        active_index: 0,
    };

    assert_eq!(
        data.active_account().map(|account| account.label.as_str()),
        Some("zen")
    );
}

#[test]
fn default_global_warn_percent_is_80() {
    assert_eq!(MochiSettings::default().usage_warn_percent, 80);
}

#[test]
fn warn_percent_clamps_to_1_to_100() {
    assert_eq!(clamp_warn_percent(0), 1);
    assert_eq!(clamp_warn_percent(101), 100);
    assert_eq!(clamp_warn_percent(90), 90);
}

#[test]
fn crossing_armed_threshold_fires_once_then_disarms() {
    assert!(should_notify_threshold(85.0, 80, true));
    assert!(!should_notify_threshold(86.0, 80, false));
}

#[test]
fn dropping_below_rearms() {
    assert!(!should_notify_threshold(79.9, 80, false));
    assert!(rearmed_below_threshold(79.9, 80));
}
