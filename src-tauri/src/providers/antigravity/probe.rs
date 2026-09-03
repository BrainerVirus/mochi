//! Antigravity local language-server probe.
//!
//! Ported from CodexBar `AntigravityStatusProbe.swift` (MIT).

#[cfg(unix)]
use std::process::Command as StdCommand;
#[cfg(unix)]
use std::time::Duration;

#[cfg(unix)]
use regex::Regex;
#[cfg(unix)]
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
#[cfg(unix)]
use reqwest::Client;

#[cfg(unix)]
use super::usage_parse::{
    parse_command_model_response, parse_user_status_response, snapshot_from_models,
};
use crate::core::models::UsageSnapshot;
use crate::core::provider::{ProviderError, ProviderResult};

#[cfg(unix)]
const GET_USER_STATUS: &str = "/exa.language_server_pb.LanguageServerService/GetUserStatus";
#[cfg(unix)]
const GET_COMMAND_MODEL_CONFIGS: &str =
    "/exa.language_server_pb.LanguageServerService/GetCommandModelConfigs";
#[cfg(unix)]
const GET_UNLEASH_DATA: &str = "/exa.language_server_pb.LanguageServerService/GetUnleashData";

#[cfg(unix)]
#[derive(Debug, Clone)]
struct ProcessInfo {
    csrf_token: String,
    extension_port: Option<u16>,
    extension_csrf_token: Option<String>,
    listening_ports: Vec<u16>,
}

pub async fn fetch_local_usage() -> ProviderResult<UsageSnapshot> {
    #[cfg(not(unix))]
    {
        return Err(ProviderError::NotConfigured);
    }

    #[cfg(unix)]
    {
        fetch_local_usage_unix().await
    }
}

#[cfg(unix)]
async fn fetch_local_usage_unix() -> ProviderResult<UsageSnapshot> {
    let process = detect_process_info()?;
    let ports = list_listening_ports(process.listening_ports.clone())?;
    let endpoint = resolve_working_endpoint(&process, &ports).await?;
    let body = default_request_body();

    match post_probe(&endpoint, GET_USER_STATUS, &body).await {
        Ok(response) => {
            let (models, _, _) = parse_user_status_response(&response)?;
            snapshot_from_models(
                &models,
                &crate::core::usage_store::current_timestamp(),
                "antigravity-local-probe",
            )
        }
        Err(_) => {
            let models = parse_command_model_response(
                &post_probe(&endpoint, GET_COMMAND_MODEL_CONFIGS, &body).await?,
            )?;
            snapshot_from_models(
                &models,
                &crate::core::usage_store::current_timestamp(),
                "antigravity-local-probe",
            )
        }
    }
}

#[cfg(unix)]
fn detect_process_info() -> ProviderResult<ProcessInfo> {
    let output = StdCommand::new("ps")
        .args(["-ax", "-o", "pid=,command="])
        .output()
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if !is_antigravity_language_server(line) {
            continue;
        }
        let csrf_token = extract_flag(line, "--csrf_token")
            .ok_or_else(|| ProviderError::Auth("Antigravity CSRF token not found".into()))?;
        let extension_port =
            extract_flag(line, "--extension_server_port").and_then(|value| value.parse().ok());
        let extension_csrf_token = extract_flag(line, "--extension_server_csrf_token");
        let pid = line
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| ProviderError::Fetch("Antigravity PID parse failed".into()))?;
        let listening_ports = list_ports_for_pid(pid)?;
        return Ok(ProcessInfo {
            csrf_token,
            extension_port,
            extension_csrf_token,
            listening_ports,
        });
    }

    Err(ProviderError::NotConfigured)
}

#[cfg(unix)]
fn list_ports_for_pid(pid: u32) -> ProviderResult<Vec<u16>> {
    let output = StdCommand::new("lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN", "-p", &pid.to_string()])
        .output()
        .map_err(|_| ProviderError::Fetch("lsof not available".into()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let port_re = Regex::new(r":(\d+)\s+\(LISTEN\)").expect("port regex");
    let mut ports = Vec::new();
    for caps in stdout.lines().filter_map(|line| port_re.captures(line)) {
        if let Some(port) = caps.get(1).and_then(|m| m.as_str().parse().ok()) {
            ports.push(port);
        }
    }
    if ports.is_empty() {
        return Err(ProviderError::Fetch(
            "Antigravity is running but not exposing ports yet".into(),
        ));
    }
    Ok(ports)
}

#[cfg(unix)]
fn list_listening_ports(mut ports: Vec<u16>) -> ProviderResult<Vec<u16>> {
    ports.sort_unstable();
    ports.dedup();
    if ports.is_empty() {
        Err(ProviderError::Fetch("no listening ports found".into()))
    } else {
        Ok(ports)
    }
}

#[cfg(unix)]
async fn resolve_working_endpoint(
    process: &ProcessInfo,
    ports: &[u16],
) -> ProviderResult<ProbeEndpoint> {
    let candidates = connection_candidates(process, ports);
    let client = localhost_client()?;
    let body = default_request_body();

    for endpoint in candidates {
        let url = format!(
            "{}://127.0.0.1:{}{}",
            endpoint.scheme, endpoint.port, GET_UNLEASH_DATA
        );
        let headers = probe_headers(&endpoint.csrf_token)?;
        if let Ok(response) = client
            .post(url)
            .headers(headers.clone())
            .json(&body)
            .send()
            .await
        {
            if response.status().is_success() {
                return Ok(endpoint);
            }
        }
    }

    Err(ProviderError::Fetch(
        "Antigravity connect port probe failed".into(),
    ))
}

#[cfg(unix)]
async fn post_probe(
    endpoint: &ProbeEndpoint,
    path: &str,
    body: &serde_json::Value,
) -> ProviderResult<String> {
    let client = localhost_client()?;
    let url = format!("{}://127.0.0.1:{}{}", endpoint.scheme, endpoint.port, path);
    let response = client
        .post(url)
        .headers(probe_headers(&endpoint.csrf_token)?)
        .json(body)
        .send()
        .await
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;
    if !response.status().is_success() {
        return Err(ProviderError::Fetch(format!(
            "Antigravity API HTTP {}",
            response.status()
        )));
    }
    response
        .text()
        .await
        .map_err(|error| ProviderError::Fetch(error.to_string()))
}

#[cfg(unix)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProbeEndpoint {
    scheme: &'static str,
    port: u16,
    csrf_token: String,
}

#[cfg(unix)]
fn connection_candidates(process: &ProcessInfo, ports: &[u16]) -> Vec<ProbeEndpoint> {
    let mut candidates = Vec::new();
    for port in ports {
        candidates.push(ProbeEndpoint {
            scheme: "https",
            port: *port,
            csrf_token: process.csrf_token.clone(),
        });
    }
    if let Some(port) = process.extension_port {
        if let Some(token) = process.extension_csrf_token.clone() {
            candidates.push(ProbeEndpoint {
                scheme: "http",
                port,
                csrf_token: token,
            });
        }
        candidates.push(ProbeEndpoint {
            scheme: "http",
            port,
            csrf_token: process.csrf_token.clone(),
        });
    }
    candidates
}

#[cfg(unix)]
fn default_request_body() -> serde_json::Value {
    serde_json::json!({
        "metadata": {
            "ideName": "antigravity",
            "extensionName": "antigravity",
            "locale": "en",
            "ideVersion": "unknown"
        }
    })
}

#[cfg(unix)]
fn probe_headers(csrf_token: &str) -> ProviderResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Codeium-Csrf-Token",
        HeaderValue::from_str(csrf_token)
            .map_err(|error| ProviderError::Fetch(error.to_string()))?,
    );
    headers.insert("Connect-Protocol-Version", HeaderValue::from_static("1"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(headers)
}

#[cfg(unix)]
fn localhost_client() -> ProviderResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(8))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|error| ProviderError::Fetch(error.to_string()))
}

#[cfg(unix)]
fn is_antigravity_language_server(line: &str) -> bool {
    let lowered = line.to_lowercase();
    if !lowered.contains("language_server") {
        return false;
    }
    lowered.contains("--app_data_dir antigravity") || lowered.contains("/antigravity/")
}

#[cfg(unix)]
fn extract_flag(line: &str, flag: &str) -> Option<String> {
    let pattern = format!(r"{flag}\s+(\S+)");
    Regex::new(&pattern)
        .ok()?
        .captures(line)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

pub fn is_probe_available() -> bool {
    #[cfg(unix)]
    {
        detect_process_info().is_ok()
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn detects_antigravity_language_server_command_line() {
        let line = "/Applications/Antigravity.app/Contents/Resources/bin/language_server_macos \
            --csrf_token token --app_data_dir antigravity";
        assert!(is_antigravity_language_server(line));
    }

    #[test]
    fn ignores_non_language_server_helpers() {
        let line = "/Applications/Antigravity.app/Contents/Frameworks/Antigravity Helper";
        assert!(!is_antigravity_language_server(line));
    }
}
