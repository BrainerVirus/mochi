use async_trait::async_trait;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE, ORIGIN, REFERER};

use super::usage_parse::{parse_auth_me, parse_usage_response, ParsedFactoryUsage};
use crate::core::provider::{ProviderError, ProviderResult};

const BASE_URLS: [&str; 3] = [
    "https://app.factory.ai",
    "https://api.factory.ai",
    "https://auth.factory.ai",
];

#[async_trait]
pub trait FactoryWebClient: Send + Sync {
    async fn fetch_usage(
        &self,
        cookie_header: &str,
        bearer_token: Option<&str>,
    ) -> ProviderResult<ParsedFactoryUsage>;
}

pub struct HttpFactoryWebClient {
    http: reqwest::Client,
}

impl HttpFactoryWebClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }
}

impl Default for HttpFactoryWebClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FactoryWebClient for HttpFactoryWebClient {
    async fn fetch_usage(
        &self,
        cookie_header: &str,
        bearer_token: Option<&str>,
    ) -> ProviderResult<ParsedFactoryUsage> {
        let mut last_error = ProviderError::Fetch("Factory API unreachable".into());

        for base_url in BASE_URLS {
            match self
                .fetch_from_base(base_url, cookie_header, bearer_token)
                .await
            {
                Ok(parsed) => return Ok(parsed),
                Err(error) => last_error = error,
            }
        }

        Err(last_error)
    }
}

impl HttpFactoryWebClient {
    async fn fetch_from_base(
        &self,
        base_url: &str,
        cookie_header: &str,
        bearer_token: Option<&str>,
    ) -> ProviderResult<ParsedFactoryUsage> {
        let auth_body = self
            .get_json(
                &format!("{base_url}/api/app/auth/me"),
                cookie_header,
                bearer_token,
            )
            .await?;
        let (email, organization, tier, plan) = parse_auth_me(&auth_body)?;

        let usage_body = self
            .post_json(
                &format!("{base_url}/api/organization/subscription/usage"),
                cookie_header,
                bearer_token,
                &serde_json::json!({ "useCache": true }),
            )
            .await?;
        let mut parsed = parse_usage_response(&usage_body)?;
        parsed.account_email = email;
        parsed.organization_name = organization;
        parsed.tier = tier;
        parsed.plan_name = plan;
        Ok(parsed)
    }

    async fn get_json(
        &self,
        url: &str,
        cookie_header: &str,
        bearer_token: Option<&str>,
    ) -> ProviderResult<String> {
        let response = self
            .http
            .get(url)
            .headers(factory_headers(cookie_header, bearer_token)?)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;
        self.response_body(response).await
    }

    async fn post_json(
        &self,
        url: &str,
        cookie_header: &str,
        bearer_token: Option<&str>,
        body: &serde_json::Value,
    ) -> ProviderResult<String> {
        let response = self
            .http
            .post(url)
            .headers(factory_headers(cookie_header, bearer_token)?)
            .json(body)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;
        self.response_body(response).await
    }

    async fn response_body(&self, response: reqwest::Response) -> ProviderResult<String> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Auth("Factory session expired".into()));
        }
        if !status.is_success() {
            return Err(ProviderError::Fetch(format!("Factory API HTTP {status}")));
        }
        Ok(body)
    }
}

fn factory_headers(
    cookie_header: &str,
    bearer_token: Option<&str>,
) -> ProviderResult<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(ORIGIN, "https://app.factory.ai".parse().unwrap());
    headers.insert(REFERER, "https://app.factory.ai/".parse().unwrap());
    headers.insert("x-factory-client", "web-app".parse().unwrap());
    headers.insert(
        COOKIE,
        cookie_header
            .parse()
            .map_err(|error: reqwest::header::InvalidHeaderValue| {
                ProviderError::Fetch(error.to_string())
            })?,
    );
    if let Some(token) = bearer_token.filter(|value| !value.is_empty()) {
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {token}").parse().map_err(
                |error: reqwest::header::InvalidHeaderValue| {
                    ProviderError::Fetch(error.to_string())
                },
            )?,
        );
    }
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::super::usage_parse::{parse_auth_me, parse_usage_response};
    use super::*;
    use async_trait::async_trait;

    struct MockFactoryWebClient;

    #[async_trait]
    impl FactoryWebClient for MockFactoryWebClient {
        async fn fetch_usage(
            &self,
            _cookie_header: &str,
            _bearer_token: Option<&str>,
        ) -> ProviderResult<ParsedFactoryUsage> {
            let (_, organization, tier, plan) =
                parse_auth_me(include_str!("fixtures/auth_me.json"))?;
            let mut parsed = parse_usage_response(include_str!("fixtures/usage.json"))?;
            parsed.organization_name = organization;
            parsed.tier = tier;
            parsed.plan_name = plan;
            Ok(parsed)
        }
    }

    #[tokio::test]
    async fn mock_client_returns_parsed_usage() {
        let client = MockFactoryWebClient;
        let parsed = client
            .fetch_usage("session=test", None)
            .await
            .expect("fetch");
        assert_eq!(parsed.standard_used_percent, 10.0);
        assert_eq!(parsed.premium_used_percent, 50.0);
    }
}
