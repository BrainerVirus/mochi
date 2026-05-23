//! Claude Web API strategy (browser `sessionKey` cookie).
//!
//! Derived from CodexBar `ClaudeWebAPIFetcher` (MIT).
//! Set `MOCHI_CLAUDE_SESSION_KEY`, `MOCHI_CLAUDE_COOKIE`, or `MOCHI_CLAUDE_COOKIE_FILE`.

mod client;
mod credentials;
mod strategy;

pub(crate) use credentials::resolve_session_key;
pub use strategy::WebStrategy;
