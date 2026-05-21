use std::sync::Arc;

use crate::core::provider::Provider;

#[derive(Default)]
pub struct ProviderRegistry {
    providers: Vec<Arc<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new(providers: Vec<Arc<dyn Provider>>) -> Self {
        Self { providers }
    }

    pub fn providers(&self) -> &[Arc<dyn Provider>] {
        &self.providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_is_valid() {
        let registry = ProviderRegistry::default();
        assert!(registry.providers().is_empty());
    }
}
