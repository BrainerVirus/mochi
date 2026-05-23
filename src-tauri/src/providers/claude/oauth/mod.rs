//! Claude OAuth usage strategy (Anthropic OAuth API).
//!
//! Derived from CodexBar `ClaudeOAuthUsageFetcher` (MIT).

mod client;
pub mod credentials;
mod strategy;

pub use credentials::current_timestamp;
pub use strategy::OAuthStrategy;
