pub(crate) mod claude;
pub(crate) mod codex;
pub(crate) mod copilot;
pub mod credential_probe;
pub(crate) mod cursor;
pub(crate) mod gemini;
pub(crate) mod opencode;
pub(crate) mod opencodego;
pub(crate) mod zai;
mod static_provider;

use std::sync::Arc;

use crate::core::models::ProviderId;
use crate::core::provider::Provider;
pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use copilot::CopilotProvider;
pub use cursor::CursorProvider;
pub use gemini::GeminiProvider;
pub use opencode::OpenCodeProvider;
pub use opencodego::OpenCodeGoProvider;
pub use zai::ZaiProvider;
use static_provider::StaticProvider;

pub fn built_in_providers() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(CodexProvider),
        Arc::new(ClaudeProvider),
        Arc::new(CursorProvider),
        Arc::new(GeminiProvider),
        Arc::new(CopilotProvider),
        Arc::new(OpenCodeProvider),
        Arc::new(OpenCodeGoProvider),
        Arc::new(StaticProvider::new(ProviderId::Antigravity, "Antigravity")),
        Arc::new(StaticProvider::new(ProviderId::Factory, "Factory/Droid")),
        Arc::new(ZaiProvider),
        Arc::new(StaticProvider::new(ProviderId::Kiro, "Kiro")),
        Arc::new(StaticProvider::new(ProviderId::Augment, "Augment")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_twelve_v1_providers() {
        assert_eq!(built_in_providers().len(), 12);
    }
}
