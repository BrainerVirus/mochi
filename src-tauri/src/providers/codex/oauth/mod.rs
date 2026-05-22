//! Codex OAuth fetch strategy.
//!
//! Token read/refresh and usage API logic derived from
//! [CodexBar](https://github.com/steipete/CodexBar) (`docs/codex-oauth.md`, MIT license).

mod client;
mod credentials;
mod parse;
mod strategy;

pub use strategy::OAuthStrategy;
