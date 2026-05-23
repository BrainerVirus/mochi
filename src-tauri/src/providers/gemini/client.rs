//! HTTP client for Gemini OAuth-backed quota APIs.
//!
//! Derived from CodexBar `GeminiStatusProbe.swift` and `docs/gemini.md` (MIT).

use std::time::Duration;

use async_trait::async_trait;
use time::OffsetDateTime;

use super::credentials::{
    credentials_path, load_credentials, update_stored_credentials, validate_auth_type,
    GeminiOAuthCredentials,
};
use super::oauth_client::{resolve_oauth_client_credentials, OAuthClientCredentials};
use super::usage_parse::{
    extract_jwt_claims, parse_code_assist_project_id, parse_quota_response, parse_tier_id,
    plan_label, snapshot_from_quotas, GeminiUserTier,
};
use crate::core::models::UsageSnapshot;
use crate::core::provider::{ProviderError, ProviderResult};

const QUOTA_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuota";
const LOAD_CODE_ASSIST_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
const PROJECTS_URL: &str = "https://cloudresourcemanager.googleapis.com/v1/projects";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[async_trait]
pub trait GeminiQuotaClient: Send + Sync {
    async fn fetch_snapshot(&self, updated_at: &str, source: &str)
        -> ProviderResult<UsageSnapshot>;
}

pub struct HttpGeminiQuotaClient {
    http: reqwest::Client,
}

impl HttpGeminiQuotaClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { http }
    }

    pub async fn ensure_access_token(
        &self,
        creds: &mut GeminiOAuthCredentials,
    ) -> ProviderResult<String> {
        if !creds.needs_refresh() {
            return creds
                .access_token
                .clone()
                .ok_or_else(|| ProviderError::Auth("gemini not logged in".into()));
        }

        let refresh_token = creds
            .refresh_token
            .clone()
            .filter(|token| !token.is_empty())
            .ok_or(ProviderError::Auth("gemini not logged in".into()))?;

        let oauth_client = resolve_oauth_client_credentials()?;
        let refresh_response = self
            .refresh_access_token(&oauth_client, &refresh_token)
            .await?;
        update_stored_credentials(&credentials_path(), &refresh_response)?;

        let access_token = refresh_response
            .get("access_token")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .ok_or_else(|| ProviderError::Auth("gemini token refresh failed".into()))?;

        creds.access_token = Some(access_token.clone());
        creds.id_token = refresh_response
            .get("id_token")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .or_else(|| creds.id_token.clone());

        Ok(access_token)
    }

    async fn refresh_access_token(
        &self,
        oauth_client: &OAuthClientCredentials,
        refresh_token: &str,
    ) -> ProviderResult<serde_json::Value> {
        let params = [
            ("client_id", oauth_client.client_id.as_str()),
            ("client_secret", oauth_client.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self
            .http
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        if response.status().as_u16() != 200 {
            return Err(ProviderError::Auth("gemini not logged in".into()));
        }

        response
            .json()
            .await
            .map_err(|error| ProviderError::Parse(error.to_string()))
    }

    async fn load_code_assist(&self, access_token: &str) -> CodeAssistStatus {
        let Ok(response) = self
            .http
            .post(LOAD_CODE_ASSIST_URL)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .body(r#"{"metadata":{"ideType":"GEMINI_CLI","pluginType":"GEMINI"}}"#)
            .send()
            .await
        else {
            return CodeAssistStatus::default();
        };

        if response.status().as_u16() != 200 {
            return CodeAssistStatus::default();
        }

        let Ok(body) = response.text().await else {
            return CodeAssistStatus::default();
        };

        CodeAssistStatus {
            tier: parse_tier_id(&body),
            project_id: parse_code_assist_project_id(&body),
        }
    }

    async fn discover_project_id(&self, access_token: &str) -> Option<String> {
        let response = self
            .http
            .get(PROJECTS_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .ok()?;

        if response.status().as_u16() != 200 {
            return None;
        }

        let body = response.text().await.ok()?;
        let json: serde_json::Value = serde_json::from_str(&body).ok()?;
        let projects = json.get("projects")?.as_array()?;

        for project in projects {
            let project_id = project.get("projectId")?.as_str()?;
            if project_id.starts_with("gen-lang-client") {
                return Some(project_id.to_string());
            }

            if project
                .get("labels")
                .and_then(|labels| labels.get("generative-language"))
                .is_some()
            {
                return Some(project_id.to_string());
            }
        }

        None
    }

    async fn fetch_quota_body(
        &self,
        access_token: &str,
        project_id: Option<&str>,
    ) -> ProviderResult<String> {
        let body = match project_id {
            Some(project_id) => serde_json::json!({ "project": project_id }).to_string(),
            None => "{}".to_string(),
        };

        let response = self
            .http
            .post(QUOTA_URL)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        match response.status().as_u16() {
            401 => Err(ProviderError::Auth("gemini not logged in".into())),
            200 => response
                .text()
                .await
                .map_err(|error| ProviderError::Fetch(error.to_string())),
            code => Err(ProviderError::Fetch(format!(
                "gemini quota request failed: HTTP {code}"
            ))),
        }
    }
}

impl Default for HttpGeminiQuotaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
struct CodeAssistStatus {
    tier: Option<GeminiUserTier>,
    project_id: Option<String>,
}

#[async_trait]
impl GeminiQuotaClient for HttpGeminiQuotaClient {
    async fn fetch_snapshot(
        &self,
        updated_at: &str,
        source: &str,
    ) -> ProviderResult<UsageSnapshot> {
        validate_auth_type(super::credentials::current_auth_type())?;
        let mut creds = load_credentials()?;
        let access_token = self.ensure_access_token(&mut creds).await?;

        let code_assist = self.load_code_assist(&access_token).await;
        let project_id = code_assist
            .project_id
            .or(self.discover_project_id(&access_token).await);

        let quota_body = self
            .fetch_quota_body(&access_token, project_id.as_deref())
            .await?;
        let quotas = parse_quota_response(&quota_body)?;
        let claims = extract_jwt_claims(creds.id_token.as_deref());
        let _account_email = claims.email;
        let _account_plan = plan_label(code_assist.tier, claims.hosted_domain.as_deref());
        snapshot_from_quotas(&quotas, updated_at, source)
    }
}

pub fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_timestamp_is_rfc3339() {
        assert!(current_timestamp().contains('T'));
    }
}
