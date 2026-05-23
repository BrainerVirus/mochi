mod codex;
mod static_provider;

use std::sync::Arc;

use crate::core::models::ProviderId;
use crate::core::provider::Provider;
pub use codex::CodexProvider;
use static_provider::StaticProvider;

pub fn built_in_providers() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(CodexProvider),
        Arc::new(StaticProvider::new(ProviderId::Claude, "Claude")),
        Arc::new(StaticProvider::new(ProviderId::Cursor, "Cursor")),
        Arc::new(StaticProvider::new(ProviderId::Gemini, "Gemini")),
        Arc::new(StaticProvider::new(ProviderId::Copilot, "Copilot")),
        Arc::new(StaticProvider::new(ProviderId::Antigravity, "Antigravity")),
        Arc::new(StaticProvider::new(ProviderId::Factory, "Factory/Droid")),
        Arc::new(StaticProvider::new(ProviderId::Zai, "z.ai")),
        Arc::new(StaticProvider::new(ProviderId::Kiro, "Kiro")),
        Arc::new(StaticProvider::new(ProviderId::Augment, "Augment")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_ten_v1_providers() {
        assert_eq!(built_in_providers().len(), 10);
    }
}
