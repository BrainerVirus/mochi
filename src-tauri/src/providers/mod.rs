pub(crate) mod antigravity;
pub(crate) mod augment;
pub(crate) mod claude;
pub(crate) mod codex;
pub(crate) mod copilot;
pub mod credential_probe;
pub(crate) mod cursor;
pub(crate) mod factory;
pub(crate) mod gemini;
pub(crate) mod kiro;
pub(crate) mod opencode;
pub(crate) mod opencodego;
pub(crate) mod zai;

use std::sync::Arc;

use crate::core::provider::Provider;
pub use antigravity::AntigravityProvider;
pub use augment::AugmentProvider;
pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use copilot::CopilotProvider;
pub use cursor::CursorProvider;
pub use factory::FactoryProvider;
pub use gemini::GeminiProvider;
pub use kiro::KiroProvider;
pub use opencode::OpenCodeProvider;
pub use opencodego::OpenCodeGoProvider;
pub use zai::ZaiProvider;

pub fn built_in_providers() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(CodexProvider),
        Arc::new(ClaudeProvider),
        Arc::new(CursorProvider),
        Arc::new(GeminiProvider),
        Arc::new(CopilotProvider),
        Arc::new(OpenCodeProvider),
        Arc::new(OpenCodeGoProvider),
        Arc::new(AntigravityProvider),
        Arc::new(FactoryProvider),
        Arc::new(ZaiProvider),
        Arc::new(KiroProvider),
        Arc::new(AugmentProvider),
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
