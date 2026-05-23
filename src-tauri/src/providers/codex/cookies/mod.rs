//! OpenAI web dashboard cookie fetch (CodexBar-derived, MIT).
//! Reference: CodexBar `Sources/CodexBarCore/OpenAIWeb/*`.

mod client;
mod credentials;
mod parse;
mod strategy;

pub use strategy::BrowserCookiesStrategy;

#[cfg(test)]
pub use client::CodexWebDashboardClient;
