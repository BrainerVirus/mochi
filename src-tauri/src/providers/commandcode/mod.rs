//! Command Code usage provider — `api.commandcode.ai` web-session API.

mod client;
mod credentials;
mod strategy;
mod usage_parse;

pub use client::CommandCodeClient;
pub(crate) use credentials::has_credentials;
pub use credentials::resolve_session_cookie;
pub use strategy::WebStrategy;
pub use usage_parse::{
    parse_credits, parse_summary, snapshot_from_commandcode, CreditsResponse, SummaryResponse,
    WindowLimit,
};
