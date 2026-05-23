//! OpenAI web dashboard cookie fetch (CodexBar-derived, MIT).
//! Reference: CodexBar `Sources/CodexBarCore/OpenAIWeb/*`.

mod client;
mod credentials;
mod parse;
mod strategy;

pub(crate) use credentials::resolve_manual_cookie;
pub use strategy::BrowserCookiesStrategy;

#[cfg(test)]
pub use client::CodexWebDashboardClient;
