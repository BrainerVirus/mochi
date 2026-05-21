use std::borrow::Cow;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};

use super::parse::snapshot_from_rate_limits_result;

const RPC_TIMEOUT: Duration = Duration::from_secs(30);
const CLIENT_NAME: &str = "mochi";

pub struct CliRpcStrategy {
    client: Arc<dyn CodexAppServer + Send + Sync>,
}

impl CliRpcStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(SubprocessCodexAppServer::default()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn CodexAppServer + Send + Sync>) -> Self {
        Self { client }
    }
}

impl Default for CliRpcStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait CodexAppServer: Send + Sync {
    async fn is_installed(&self) -> bool;
    async fn fetch_rate_limits(&self) -> ProviderResult<Value>;
}

#[derive(Default)]
pub struct SubprocessCodexAppServer {
    binary: Option<PathBuf>,
}

impl SubprocessCodexAppServer {
    fn binary_path(&self) -> Cow<'_, str> {
        match &self.binary {
            Some(path) => path.to_string_lossy(),
            None => Cow::Borrowed("codex"),
        }
    }
}

#[async_trait]
impl CodexAppServer for SubprocessCodexAppServer {
    async fn is_installed(&self) -> bool {
        Command::new(self.binary_path().as_ref())
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    async fn fetch_rate_limits(&self) -> ProviderResult<Value> {
        timeout(RPC_TIMEOUT, run_codex_rate_limits_rpc(self.binary_path().as_ref()))
            .await
            .map_err(|_| ProviderError::Timeout)?
    }
}

#[async_trait]
impl FetchStrategy for CliRpcStrategy {
    fn id(&self) -> &'static str {
        "codex-cli-rpc"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::Cli
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(self.client.is_installed().await)
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let result = self.client.fetch_rate_limits().await?;
        let updated_at = current_timestamp();
        snapshot_from_rate_limits_result(&result, &updated_at)
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::NotConfigured | ProviderError::Auth(_) | ProviderError::Timeout
        )
    }
}

async fn run_codex_rate_limits_rpc(binary: &str) -> ProviderResult<Value> {
    let mut child = spawn_app_server(binary)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ProviderError::Fetch("codex app-server stdout unavailable".into()))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| ProviderError::Fetch("codex app-server stdin unavailable".into()))?;

    let mut reader = BufReader::new(stdout).lines();

    send_rpc_message(
        &mut stdin,
        json!({
            "method": "initialize",
            "id": 0,
            "params": {
                "clientInfo": {
                    "name": CLIENT_NAME,
                    "title": "Mochi",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }
        }),
    )
    .await?;

    let initialize_response = read_rpc_response(&mut reader, 0).await?;
    if initialize_response.get("error").is_some() {
        return Err(map_rpc_error(&initialize_response));
    }

    send_rpc_message(
        &mut stdin,
        json!({
            "method": "initialized",
            "params": {}
        }),
    )
    .await?;

    send_rpc_message(
        &mut stdin,
        json!({
            "method": "account/rateLimits/read",
            "id": 1
        }),
    )
    .await?;

    let rate_limits_response = read_rpc_response(&mut reader, 1).await?;
    let _ = child.kill().await;

    rate_limits_response
        .get("result")
        .cloned()
        .ok_or_else(|| map_rpc_error(&rate_limits_response))
}

fn spawn_app_server(binary: &str) -> ProviderResult<Child> {
    Command::new(binary)
        .args(["app-server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ProviderError::NotConfigured
            } else {
                ProviderError::Fetch(error.to_string())
            }
        })
}

async fn send_rpc_message(
    stdin: &mut tokio::process::ChildStdin,
    message: Value,
) -> ProviderResult<()> {
    let mut payload = serde_json::to_string(&message)
        .map_err(|error| ProviderError::Parse(error.to_string()))?;
    payload.push('\n');
    stdin
        .write_all(payload.as_bytes())
        .await
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;
    stdin
        .flush()
        .await
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;
    Ok(())
}

async fn read_rpc_response(
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    id: u64,
) -> ProviderResult<Value> {
    while let Some(line) = reader
        .next_line()
        .await
        .map_err(|error| ProviderError::Fetch(error.to_string()))?
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let message: Value = serde_json::from_str(trimmed)
            .map_err(|error| ProviderError::Parse(error.to_string()))?;

        if message.get("method").is_some() {
            continue;
        }

        if message.get("id").and_then(Value::as_u64) == Some(id) {
            return Ok(message);
        }
    }

    Err(ProviderError::Fetch(
        "codex app-server closed before response".into(),
    ))
}

fn map_rpc_error(response: &Value) -> ProviderError {
    let message = response
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("codex app-server request failed");

    if message.contains("Not initialized") || message.contains("auth") {
        ProviderError::Auth(message.to_string())
    } else {
        ProviderError::Fetch(message.to_string())
    }
}

fn current_timestamp() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCodexAppServer {
        installed: bool,
        result: ProviderResult<Value>,
    }

    #[async_trait]
    impl CodexAppServer for MockCodexAppServer {
        async fn is_installed(&self) -> bool {
            self.installed
        }

        async fn fetch_rate_limits(&self) -> ProviderResult<Value> {
            match &self.result {
                Ok(value) => Ok(value.clone()),
                Err(error) => Err(match error {
                    ProviderError::NotConfigured => ProviderError::NotConfigured,
                    ProviderError::Timeout => ProviderError::Timeout,
                    ProviderError::Auth(message) => ProviderError::Auth(message.clone()),
                    ProviderError::Parse(message) => ProviderError::Parse(message.clone()),
                    ProviderError::Fetch(message) => ProviderError::Fetch(message.clone()),
                }),
            }
        }
    }

    #[tokio::test]
    async fn fetch_uses_mock_rate_limits_payload() {
        let result: Value = serde_json::from_str(include_str!("fixtures/rate_limits.json"))
            .expect("fixture json");
        let strategy = CliRpcStrategy::with_client(Arc::new(MockCodexAppServer {
            installed: true,
            result: Ok(result),
        }));

        let snapshot = strategy
            .fetch(&FetchContext)
            .await
            .expect("mock fetch should succeed");

        assert_eq!(snapshot.source, "codex-cli");
        assert_eq!(snapshot.primary.used_percent, 25.0);
    }

    #[tokio::test]
    async fn is_unavailable_when_codex_binary_missing() {
        let strategy = CliRpcStrategy::with_client(Arc::new(MockCodexAppServer {
            installed: false,
            result: Err(ProviderError::NotConfigured),
        }));

        let available = strategy
            .is_available(&FetchContext)
            .await
            .expect("availability check should succeed");

        assert!(!available);
    }
}
