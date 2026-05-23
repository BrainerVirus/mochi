//! Codex OAuth fetch strategy.
//!
//! Token read/refresh and usage API logic derived from
//! [CodexBar](https://github.com/steipete/CodexBar) (`docs/codex-oauth.md`, MIT license).

mod client;
mod credentials;
mod parse;
mod strategy;

pub(crate) use credentials::{codex_auth_path, load_credentials_from_path};
pub use strategy::OAuthStrategy;
