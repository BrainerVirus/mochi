use crate::core::models::ProviderId;
use crate::core::provider::FetchKind;

/// CodexBar-derived provider definitions for Mochi v1.
/// Reference: CodexBar `docs/providers.md` (MIT).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationStatus {
    Stub,
    Partial,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyDefinition {
    pub id: &'static str,
    pub kind: FetchKind,
    pub label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthRequirement {
    OAuth,
    ApiKey,
    BrowserCookies,
    CliSession,
    LocalProbe,
    AdminApiKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFieldKind {
    ApiKey,
    CookieSource,
    ManualCookie,
    TokenAccount,
    HistoryWindow,
    RegionHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsFieldDefinition {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: SettingsFieldKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderDefinition {
    pub id: ProviderId,
    pub codexbar_id: &'static str,
    pub display_name: &'static str,
    pub strategies: &'static [StrategyDefinition],
    pub auth_requirements: &'static [AuthRequirement],
    pub settings_fields: &'static [SettingsFieldDefinition],
    pub status_url: Option<&'static str>,
    pub supports_cost: bool,
    pub implementation_status: ImplementationStatus,
}

const COOKIE_SOURCE: SettingsFieldDefinition = SettingsFieldDefinition {
    key: "cookie_source",
    label: "Cookie source",
    kind: SettingsFieldKind::CookieSource,
};

const MANUAL_COOKIE: SettingsFieldDefinition = SettingsFieldDefinition {
    key: "manual_cookie",
    label: "Manual cookie header",
    kind: SettingsFieldKind::ManualCookie,
};

const HISTORY_WINDOW: SettingsFieldDefinition = SettingsFieldDefinition {
    key: "history_window_days",
    label: "Session cost history window",
    kind: SettingsFieldKind::HistoryWindow,
};

const CODEX: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Codex,
    codexbar_id: "codex",
    display_name: "Codex",
    strategies: &[
        StrategyDefinition {
            id: "codex-oauth",
            kind: FetchKind::OAuth,
            label: "OAuth API",
        },
        StrategyDefinition {
            id: "codex-cli-rpc",
            kind: FetchKind::Cli,
            label: "CLI RPC",
        },
        StrategyDefinition {
            id: "codex-browser-cookies",
            kind: FetchKind::BrowserCookies,
            label: "Web dashboard",
        },
    ],
    auth_requirements: &[AuthRequirement::OAuth, AuthRequirement::BrowserCookies, AuthRequirement::CliSession],
    settings_fields: &[COOKIE_SOURCE, HISTORY_WINDOW],
    status_url: Some("https://status.openai.com"),
    supports_cost: true,
    implementation_status: ImplementationStatus::Partial,
};

const CLAUDE: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Claude,
    codexbar_id: "claude",
    display_name: "Claude",
    strategies: &[
        StrategyDefinition {
            id: "claude-admin-api",
            kind: FetchKind::ApiKey,
            label: "Admin API",
        },
        StrategyDefinition {
            id: "claude-oauth",
            kind: FetchKind::OAuth,
            label: "OAuth API",
        },
        StrategyDefinition {
            id: "claude-cli",
            kind: FetchKind::Cli,
            label: "CLI PTY",
        },
        StrategyDefinition {
            id: "claude-web",
            kind: FetchKind::BrowserCookies,
            label: "Web API",
        },
    ],
    auth_requirements: &[
        AuthRequirement::AdminApiKey,
        AuthRequirement::OAuth,
        AuthRequirement::CliSession,
        AuthRequirement::BrowserCookies,
    ],
    settings_fields: &[SettingsFieldDefinition {
        key: "admin_api_key",
        label: "Admin API key",
        kind: SettingsFieldKind::ApiKey,
    }, COOKIE_SOURCE, HISTORY_WINDOW],
    status_url: Some("https://status.anthropic.com"),
    supports_cost: true,
    implementation_status: ImplementationStatus::Stub,
};

const CURSOR: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Cursor,
    codexbar_id: "cursor",
    display_name: "Cursor",
    strategies: &[StrategyDefinition {
        id: "cursor-web",
        kind: FetchKind::BrowserCookies,
        label: "Web API",
    }],
    auth_requirements: &[AuthRequirement::BrowserCookies],
    settings_fields: &[COOKIE_SOURCE, MANUAL_COOKIE],
    status_url: Some("https://status.cursor.com"),
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const GEMINI: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Gemini,
    codexbar_id: "gemini",
    display_name: "Gemini",
    strategies: &[StrategyDefinition {
        id: "gemini-oauth-quota",
        kind: FetchKind::OAuth,
        label: "Gemini CLI OAuth",
    }],
    auth_requirements: &[AuthRequirement::OAuth, AuthRequirement::CliSession],
    settings_fields: &[],
    status_url: Some("https://www.google.com/appsstatus/dashboard/incidents"),
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const COPILOT: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Copilot,
    codexbar_id: "copilot",
    display_name: "Copilot",
    strategies: &[StrategyDefinition {
        id: "copilot-oauth-internal",
        kind: FetchKind::OAuth,
        label: "GitHub device flow",
    }],
    auth_requirements: &[AuthRequirement::OAuth],
    settings_fields: &[SettingsFieldDefinition {
        key: "token_accounts",
        label: "Token accounts",
        kind: SettingsFieldKind::TokenAccount,
    }],
    status_url: Some("https://www.githubstatus.com"),
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const ANTIGRAVITY: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Antigravity,
    codexbar_id: "antigravity",
    display_name: "Antigravity",
    strategies: &[StrategyDefinition {
        id: "antigravity-local-probe",
        kind: FetchKind::LocalProbe,
        label: "Local LSP probe",
    }],
    auth_requirements: &[AuthRequirement::LocalProbe],
    settings_fields: &[],
    status_url: Some("https://www.google.com/appsstatus/dashboard/incidents"),
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const FACTORY: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Factory,
    codexbar_id: "factory",
    display_name: "Factory/Droid",
    strategies: &[
        StrategyDefinition {
            id: "factory-web-cookies",
            kind: FetchKind::BrowserCookies,
            label: "Web cookies",
        },
        StrategyDefinition {
            id: "factory-local-storage",
            kind: FetchKind::LocalConfig,
            label: "Local storage",
        },
    ],
    auth_requirements: &[AuthRequirement::BrowserCookies, AuthRequirement::OAuth],
    settings_fields: &[COOKIE_SOURCE, MANUAL_COOKIE],
    status_url: Some("https://status.factory.ai"),
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const ZAI: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Zai,
    codexbar_id: "zai",
    display_name: "z.ai",
    strategies: &[StrategyDefinition {
        id: "zai-api-quota",
        kind: FetchKind::ApiKey,
        label: "API quota",
    }],
    auth_requirements: &[AuthRequirement::ApiKey],
    settings_fields: &[SettingsFieldDefinition {
        key: "api_key",
        label: "API key",
        kind: SettingsFieldKind::ApiKey,
    }, SettingsFieldDefinition {
        key: "region_host",
        label: "Region host",
        kind: SettingsFieldKind::RegionHost,
    }],
    status_url: None,
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const KIRO: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Kiro,
    codexbar_id: "kiro",
    display_name: "Kiro",
    strategies: &[StrategyDefinition {
        id: "kiro-cli-usage",
        kind: FetchKind::Cli,
        label: "kiro-cli /usage",
    }],
    auth_requirements: &[AuthRequirement::CliSession],
    settings_fields: &[],
    status_url: Some("https://health.aws.amazon.com/health/status"),
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const AUGMENT: ProviderDefinition = ProviderDefinition {
    id: ProviderId::Augment,
    codexbar_id: "augment",
    display_name: "Augment",
    strategies: &[
        StrategyDefinition {
            id: "augment-cli",
            kind: FetchKind::Cli,
            label: "auggie CLI",
        },
        StrategyDefinition {
            id: "augment-web",
            kind: FetchKind::BrowserCookies,
            label: "Web cookies",
        },
    ],
    auth_requirements: &[AuthRequirement::CliSession, AuthRequirement::BrowserCookies],
    settings_fields: &[COOKIE_SOURCE, MANUAL_COOKIE],
    status_url: None,
    supports_cost: false,
    implementation_status: ImplementationStatus::Stub,
};

const REGISTRY: [ProviderDefinition; 10] = [
    CODEX, CLAUDE, CURSOR, GEMINI, COPILOT, ANTIGRAVITY, FACTORY, ZAI, KIRO, AUGMENT,
];

pub fn provider_registry() -> &'static [ProviderDefinition] {
    &REGISTRY
}

pub fn definition_for(id: ProviderId) -> Option<&'static ProviderDefinition> {
    REGISTRY.iter().find(|definition| definition.id == id)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn registry_contains_all_v1_providers() {
        let ids: HashSet<_> = provider_registry().iter().map(|definition| definition.id).collect();
        for id in ProviderId::all() {
            assert!(ids.contains(id), "missing metadata for {id:?}");
        }
        assert_eq!(ids.len(), ProviderId::all().len());
    }

    #[test]
    fn codex_is_only_partial_implementation() {
        let codex = definition_for(ProviderId::Codex).expect("codex definition");
        assert_eq!(codex.implementation_status, ImplementationStatus::Partial);
        assert!(codex.supports_cost);
    }

    #[test]
    fn stub_providers_have_fetch_strategies_declared() {
        for definition in provider_registry() {
            if definition.implementation_status == ImplementationStatus::Stub {
                assert!(
                    !definition.strategies.is_empty(),
                    "{:?} stub must declare strategies",
                    definition.id
                );
            }
        }
    }
}
