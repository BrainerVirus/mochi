use std::collections::HashMap;
use std::sync::RwLock;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CredentialError {
    #[error("credential not found: {0}")]
    NotFound(String),
    #[error("credential store unavailable: {0}")]
    Unavailable(String),
    #[error("credential store io failed: {0}")]
    Io(String),
}

pub trait CredentialStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, CredentialError>;
    fn set(&self, key: &str, value: &str) -> Result<(), CredentialError>;
    fn delete(&self, key: &str) -> Result<(), CredentialError>;
}

/// In-memory store for development and unit tests.
#[derive(Debug, Default)]
pub struct DevCredentialStore {
    inner: RwLock<HashMap<String, String>>,
}

impl DevCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for DevCredentialStore {
    fn get(&self, key: &str) -> Result<Option<String>, CredentialError> {
        let store = self
            .inner
            .read()
            .map_err(|error| CredentialError::Unavailable(error.to_string()))?;
        Ok(store.get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), CredentialError> {
        let mut store = self
            .inner
            .write()
            .map_err(|error| CredentialError::Unavailable(error.to_string()))?;
        store.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), CredentialError> {
        let mut store = self
            .inner
            .write()
            .map_err(|error| CredentialError::Unavailable(error.to_string()))?;
        store.remove(key);
        Ok(())
    }
}

pub type MemoryCredentialStore = DevCredentialStore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_store_round_trips_secrets() {
        let store = DevCredentialStore::new();
        store.set("codex.oauth", "token").unwrap();
        assert_eq!(
            store.get("codex.oauth").unwrap(),
            Some("token".to_string())
        );
        store.delete("codex.oauth").unwrap();
        assert_eq!(store.get("codex.oauth").unwrap(), None);
    }

    #[test]
    fn dev_store_get_missing_returns_none() {
        let store = DevCredentialStore::new();
        assert_eq!(store.get("missing").unwrap(), None);
    }
}
