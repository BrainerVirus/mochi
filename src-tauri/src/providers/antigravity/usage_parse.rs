//! Antigravity local LSP response parsing and window mapping.

use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone)]
pub(crate) struct ModelQuota {
    label: String,
    model_id: String,
    remaining_fraction: Option<f64>,
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserStatusResponse {
    code: Option<serde_json::Value>,
    #[serde(default, rename = "userStatus")]
    user_status: Option<UserStatusBody>,
}

#[derive(Debug, Deserialize)]
struct CommandModelConfigResponse {
    code: Option<serde_json::Value>,
    #[serde(default, rename = "clientModelConfigs")]
    client_model_configs: Vec<ModelConfig>,
}

#[derive(Debug, Deserialize)]
struct UserStatusBody {
    #[serde(default)]
    email: Option<String>,
    #[serde(default, rename = "userTier")]
    user_tier: Option<UserTier>,
    #[serde(default, rename = "planStatus")]
    plan_status: Option<PlanStatus>,
    #[serde(default, rename = "cascadeModelConfigData")]
    cascade_model_config_data: Option<CascadeModelConfigData>,
}

#[derive(Debug, Deserialize)]
struct UserTier {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlanStatus {
    #[serde(default, rename = "planInfo")]
    plan_info: Option<PlanInfo>,
}

#[derive(Debug, Deserialize)]
struct PlanInfo {
    #[serde(default, rename = "planName")]
    plan_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CascadeModelConfigData {
    #[serde(default, rename = "clientModelConfigs")]
    client_model_configs: Vec<ModelConfig>,
}

#[derive(Debug, Deserialize)]
struct ModelConfig {
    #[serde(default)]
    label: String,
    #[serde(default, rename = "modelOrAlias")]
    model_or_alias: Option<ModelOrAlias>,
    #[serde(default, rename = "quotaInfo")]
    quota_info: Option<QuotaInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelOrAlias {
    #[serde(default)]
    model: String,
}

#[derive(Debug, Deserialize)]
struct QuotaInfo {
    #[serde(default, rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(default, rename = "resetTime")]
    reset_time: Option<String>,
}

pub fn parse_user_status_response(
    body: &str,
) -> ProviderResult<(Vec<ModelQuota>, Option<String>, Option<String>)> {
    let response: UserStatusResponse =
        serde_json::from_str(body).map_err(|error| ProviderError::Parse(error.to_string()))?;
    if let Some(message) = invalid_code(response.code.as_ref()) {
        return Err(ProviderError::Auth(message));
    }
    let user_status = response
        .user_status
        .ok_or_else(|| ProviderError::Parse("Missing userStatus".into()))?;
    let models = user_status
        .cascade_model_config_data
        .map(|data| quotas_from_configs(data.client_model_configs))
        .unwrap_or_default();
    let plan = user_status
        .user_tier
        .and_then(|tier| tier.name.filter(|name| !name.trim().is_empty()))
        .or_else(|| {
            user_status
                .plan_status
                .and_then(|status| status.plan_info)
                .and_then(|info| info.plan_name)
        });
    Ok((models, user_status.email, plan))
}

pub fn parse_command_model_response(body: &str) -> ProviderResult<Vec<ModelQuota>> {
    let response: CommandModelConfigResponse =
        serde_json::from_str(body).map_err(|error| ProviderError::Parse(error.to_string()))?;
    if let Some(message) = invalid_code(response.code.as_ref()) {
        return Err(ProviderError::Auth(message));
    }
    Ok(quotas_from_configs(response.client_model_configs))
}

pub fn snapshot_from_models(
    models: &[ModelQuota],
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    if models.is_empty() {
        return Err(ProviderError::Parse("No quota models available".into()));
    }

    let primary = representative_model(models, ModelFamily::Claude)
        .or_else(|| fallback_representative(models))
        .map(usage_window_from_quota);
    let secondary =
        representative_model(models, ModelFamily::GeminiPro).map(usage_window_from_quota);
    let tertiary =
        representative_model(models, ModelFamily::GeminiFlash).map(usage_window_from_quota);

    let primary = primary.ok_or_else(|| ProviderError::Parse("No primary quota window".into()))?;
    let mut snapshot = UsageSnapshot::new(
        ProviderId::Antigravity,
        primary,
        secondary,
        updated_at,
        source,
    );
    if let Some(tertiary) = tertiary {
        snapshot = snapshot.with_tertiary(tertiary);
    }
    Ok(snapshot)
}

fn quotas_from_configs(configs: Vec<ModelConfig>) -> Vec<ModelQuota> {
    configs
        .into_iter()
        .filter_map(|config| {
            let quota = config.quota_info?;
            Some(ModelQuota {
                label: config.label,
                model_id: config
                    .model_or_alias
                    .map(|alias| alias.model)
                    .unwrap_or_default(),
                remaining_fraction: quota.remaining_fraction,
                reset_time: quota.reset_time.and_then(parse_reset_time),
            })
        })
        .collect()
}

fn usage_window_from_quota(quota: ModelQuota) -> UsageWindow {
    let remaining_percent = quota
        .remaining_fraction
        .map(|fraction| (fraction * 100.0).clamp(0.0, 100.0) as f32)
        .unwrap_or(0.0);
    let used_percent = 100.0 - remaining_percent;
    UsageWindow::new(canonical_label(&quota), used_percent, quota.reset_time)
}

fn canonical_label(model: &ModelQuota) -> String {
    match family_of(&model.model_id, &model.label) {
        ModelFamily::Claude => "Claude".into(),
        ModelFamily::GeminiPro => "Gemini Pro".into(),
        ModelFamily::GeminiFlash => "Gemini Flash".into(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModelFamily {
    Claude,
    GeminiPro,
    GeminiFlash,
}

fn representative_model(models: &[ModelQuota], family: ModelFamily) -> Option<ModelQuota> {
    let mut candidates: Vec<&ModelQuota> = models
        .iter()
        .filter(|model| family_of(&model.model_id, &model.label) == family)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|left, right| {
        let left_has = left.remaining_fraction.is_some();
        let right_has = right.remaining_fraction.is_some();
        left_has.cmp(&right_has).reverse().then_with(|| {
            remaining_percent(left)
                .partial_cmp(&remaining_percent(right))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    candidates.first().copied().cloned()
}

fn fallback_representative(models: &[ModelQuota]) -> Option<ModelQuota> {
    models
        .iter()
        .min_by(|left, right| {
            remaining_percent(left)
                .partial_cmp(&remaining_percent(right))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

fn remaining_percent(model: &ModelQuota) -> f32 {
    model
        .remaining_fraction
        .map(|fraction| (fraction * 100.0) as f32)
        .unwrap_or(0.0)
}

fn family_of(model_id: &str, label: &str) -> ModelFamily {
    let model_id = model_id.to_lowercase();
    let label = label.to_lowercase();
    if model_id.contains("claude") || label.contains("claude") {
        return ModelFamily::Claude;
    }
    if (model_id.contains("pro") && model_id.contains("low"))
        || (label.contains("pro") && label.contains("low"))
    {
        return ModelFamily::GeminiPro;
    }
    if (model_id.contains("gemini") && model_id.contains("flash"))
        || (label.contains("gemini") && label.contains("flash"))
    {
        return ModelFamily::GeminiFlash;
    }
    ModelFamily::Claude
}

fn parse_reset_time(value: String) -> Option<String> {
    if let Ok(parsed) = OffsetDateTime::parse(&value, &Rfc3339) {
        return parsed.format(&Rfc3339).ok();
    }
    if let Ok(seconds) = value.parse::<i64>() {
        return OffsetDateTime::from_unix_timestamp(seconds)
            .ok()?
            .format(&Rfc3339)
            .ok();
    }
    None
}

fn invalid_code(code: Option<&serde_json::Value>) -> Option<String> {
    let code = code?;
    if code.as_i64() == Some(0) || code.as_str() == Some("OK") {
        return None;
    }
    Some(code.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_user_status_fixture_to_claude_gemini_windows() {
        let (models, email, plan) =
            parse_user_status_response(include_str!("fixtures/user_status.json")).expect("parse");
        assert_eq!(email.as_deref(), Some("test@example.com"));
        assert_eq!(plan.as_deref(), Some("Pro"));
        assert_eq!(models.len(), 3);

        let snapshot =
            snapshot_from_models(&models, "2026-05-23T00:00:00Z", "antigravity-local-probe")
                .expect("snapshot");
        assert_eq!(snapshot.primary.label, "Claude");
        assert_eq!(snapshot.primary.used_percent, 50.0);
        assert_eq!(
            snapshot.secondary.as_ref().expect("pro").label,
            "Gemini Pro"
        );
        assert_eq!(snapshot.secondary.expect("pro").used_percent, 20.0);
        assert_eq!(
            snapshot.tertiary.as_ref().expect("flash").label,
            "Gemini Flash"
        );
        assert_eq!(snapshot.tertiary.expect("flash").used_percent, 80.0);
    }
}
