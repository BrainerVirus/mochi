use serde::{Deserialize, Serialize};

use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderUsageStateKind {
    Fresh,
    Fetching,
    StaleError,
    MissingCredentials,
    CredentialsNeedRefresh,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderUsageState {
    pub provider: ProviderId,
    pub kind: ProviderUsageStateKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<UsageSnapshot>,
    pub health: ProviderHealth,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub updated_at: String,
}

impl ProviderUsageState {
    pub fn fresh(snapshot: UsageSnapshot) -> Self {
        Self {
            provider: snapshot.provider,
            updated_at: snapshot.updated_at.clone(),
            snapshot: Some(snapshot),
            kind: ProviderUsageStateKind::Fresh,
            health: ProviderHealth::Ok,
            message: None,
        }
    }

    pub fn fetching(provider: ProviderId, updated_at: String) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::Fetching,
            snapshot: None,
            health: ProviderHealth::Stale,
            message: Some("fetching usage".into()),
            updated_at,
        }
    }

    pub fn missing_credentials(provider: ProviderId, updated_at: String) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::MissingCredentials,
            snapshot: None,
            health: ProviderHealth::Error,
            message: Some("credentials missing".into()),
            updated_at,
        }
    }

    pub fn credentials_need_refresh(provider: ProviderId) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::CredentialsNeedRefresh,
            snapshot: None,
            health: ProviderHealth::Error,
            message: Some("credentials need refresh".into()),
            updated_at: crate::core::usage_store::current_timestamp(),
        }
    }

    pub fn stale_error(mut snapshot: UsageSnapshot, message: impl Into<String>) -> Self {
        snapshot.health = ProviderHealth::Stale;
        snapshot.is_stale = true;
        let message = message.into();
        snapshot.error = Some(message.clone());

        Self {
            provider: snapshot.provider,
            updated_at: snapshot.updated_at.clone(),
            snapshot: Some(snapshot),
            kind: ProviderUsageStateKind::StaleError,
            health: ProviderHealth::Stale,
            message: Some(message),
        }
    }

    pub fn error(provider: ProviderId, message: impl Into<String>, updated_at: String) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::Error,
            snapshot: None,
            health: ProviderHealth::Error,
            message: Some(message.into()),
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot, UsageWindow};

    fn snapshot(provider: ProviderId) -> UsageSnapshot {
        UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", 42.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        )
    }

    #[test]
    fn fresh_state_wraps_successful_snapshot() {
        let state = ProviderUsageState::fresh(snapshot(ProviderId::Claude));

        assert_eq!(state.provider, ProviderId::Claude);
        assert_eq!(state.kind, ProviderUsageStateKind::Fresh);
        assert!(state.snapshot.is_some());
        assert_eq!(state.health, ProviderHealth::Ok);
        assert_eq!(state.message, None);
    }

    #[test]
    fn credentials_need_refresh_without_snapshot_has_no_meters() {
        let state = ProviderUsageState::credentials_need_refresh(ProviderId::Codex);

        assert_eq!(state.provider, ProviderId::Codex);
        assert_eq!(state.kind, ProviderUsageStateKind::CredentialsNeedRefresh);
        assert!(state.snapshot.is_none());
        assert_eq!(state.health, ProviderHealth::Error);
        assert_eq!(state.message.as_deref(), Some("credentials need refresh"));
    }

    #[test]
    fn stale_error_preserves_last_successful_snapshot() {
        let state = ProviderUsageState::stale_error(
            snapshot(ProviderId::Cursor),
            "provider fetch failed: network",
        );

        assert_eq!(state.kind, ProviderUsageStateKind::StaleError);
        assert_eq!(state.health, ProviderHealth::Stale);
        assert_eq!(
            state.message.as_deref(),
            Some("provider fetch failed: network")
        );
        assert!(state
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.is_stale));
    }
}
