//! HTTP client for OpenCode `_server` endpoints.
//!
//! Derived from CodexBar `OpenCodeUsageFetcher.swift` (MIT).

use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, COOKIE, ORIGIN, REFERER, USER_AGENT,
};

use super::credentials::ResolvedOpenCodeSession;
use super::usage_parse::{parse_subscription, parse_workspace_ids, snapshot_from_usage};
use super::zen_balance::{parse_zen_balance, zen_balance_cost};
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{ProviderError, ProviderResult};

const BASE_URL: &str = "https://opencode.ai";
const SERVER_URL: &str = "https://opencode.ai/_server";
const WORKSPACES_SERVER_ID: &str =
    "def39973159c7f0483d8793a822b8dbb10d067e12c65455fcb4608459ba0234f";
const SUBSCRIPTION_SERVER_ID: &str =
    "7abeebee372f304e050aaaf92be863f4a86490e382f8c79db68fd94040d691b4";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait OpenCodeWebClient: Send + Sync {
    async fn fetch_usage(
        &self,
        session: &ResolvedOpenCodeSession,
        provider: ProviderId,
        updated_at: &str,
        source: &str,
    ) -> ProviderResult<UsageSnapshot>;
}

pub struct HttpOpenCodeWebClient {
    http: reqwest::Client,
}

impl HttpOpenCodeWebClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }
}

impl Default for HttpOpenCodeWebClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OpenCodeWebClient for HttpOpenCodeWebClient {
    async fn fetch_usage(
        &self,
        session: &ResolvedOpenCodeSession,
        provider: ProviderId,
        updated_at: &str,
        source: &str,
    ) -> ProviderResult<UsageSnapshot> {
        let workspace_id = match session.workspace_id.as_deref() {
            Some(id) => id.to_string(),
            None => self.fetch_workspace_id(&session.cookie_header).await?,
        };
        let subscription = self
            .fetch_subscription(&session.cookie_header, &workspace_id)
            .await?;
        let data = parse_subscription(&subscription, updated_at)?;
        let mut snapshot = snapshot_from_usage(&data, provider, updated_at, source)?;

        if provider == ProviderId::OpenCodeGo {
            if let Some(balance) = self
                .fetch_optional_zen_balance(&session.cookie_header, &workspace_id)
                .await
            {
                if let Ok(cost) = zen_balance_cost(balance) {
                    snapshot = snapshot.with_provider_cost(cost);
                }
            }
        }

        Ok(snapshot)
    }
}

impl HttpOpenCodeWebClient {
    async fn fetch_optional_zen_balance(
        &self,
        cookie_header: &str,
        workspace_id: &str,
    ) -> Option<f64> {
        let url = format!("{BASE_URL}/workspace/{workspace_id}");
        let text = self.fetch_page_text(&url, cookie_header, &url).await.ok()?;
        parse_zen_balance(&text)
    }

    async fn fetch_page_text(
        &self,
        url: &str,
        cookie_header: &str,
        referer: &str,
    ) -> ProviderResult<String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(cookie_header)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(ORIGIN, HeaderValue::from_static(BASE_URL));
        headers.insert(
            REFERER,
            HeaderValue::from_str(referer)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );

        let response = self
            .http
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        if status.is_success() {
            Ok(text)
        } else {
            Err(ProviderError::Fetch(format!(
                "opencode page request failed: HTTP {}",
                status.as_u16()
            )))
        }
    }
}

impl HttpOpenCodeWebClient {
    async fn fetch_workspace_id(&self, cookie_header: &str) -> ProviderResult<String> {
        let text = self
            .fetch_server_text(WORKSPACES_SERVER_ID, "GET", None, cookie_header, BASE_URL)
            .await?;

        let mut ids = parse_workspace_ids(&text);
        if ids.is_empty() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                collect_workspace_ids(&value, &mut ids);
            }
        }

        ids.into_iter()
            .next()
            .ok_or_else(|| ProviderError::Parse("opencode missing workspace id".into()))
    }

    async fn fetch_subscription(
        &self,
        cookie_header: &str,
        workspace_id: &str,
    ) -> ProviderResult<String> {
        let referer = format!("{BASE_URL}/workspace/{workspace_id}/billing");
        self.fetch_server_text(
            SUBSCRIPTION_SERVER_ID,
            "GET",
            Some(vec![workspace_id.to_string()]),
            cookie_header,
            &referer,
        )
        .await
    }

    async fn fetch_server_text(
        &self,
        server_id: &str,
        method: &str,
        args: Option<Vec<String>>,
        cookie_header: &str,
        referer: &str,
    ) -> ProviderResult<String> {
        let mut url = reqwest::Url::parse(SERVER_URL)
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("id", server_id);
            query.append_pair("method", method);
            if let Some(args) = &args {
                for arg in args {
                    query.append_pair("args", arg);
                }
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(cookie_header)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?,
        );
        headers.insert(
            "X-Server-Id",
            HeaderValue::from_str(server_id)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?,
        );
        headers.insert(
            "X-Server-Instance",
            HeaderValue::from_str(&format!(
                "server-fn:{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| duration.as_nanos())
                    .unwrap_or(0)
            ))
            .map_err(|error| ProviderError::Fetch(error.to_string()))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(ORIGIN, HeaderValue::from_static(BASE_URL));
        headers.insert(
            REFERER,
            HeaderValue::from_str(referer)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("text/javascript, application/json;q=0.9, */*;q=0.8"),
        );

        let mut request = self
            .http
            .request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                url,
            )
            .headers(headers);
        if method != "GET" {
            if let Some(args) = args {
                request = request.header(CONTENT_TYPE, "application/json").json(&args);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        match status.as_u16() {
            200 => Ok(text),
            401 | 403 => Err(ProviderError::Auth(
                "opencode session unauthorized or expired".into(),
            )),
            code => Err(ProviderError::Fetch(format!(
                "opencode server request failed: HTTP {code}"
            ))),
        }
    }
}

fn collect_workspace_ids(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for nested in map.values() {
                collect_workspace_ids(nested, out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_workspace_ids(item, out);
            }
        }
        serde_json::Value::String(text) if text.starts_with("wrk_") && !out.contains(text) => {
            out.push(text.clone());
        }
        _ => {}
    }
}
