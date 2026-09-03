use std::time::Duration;

use async_trait::async_trait;

use crate::core::provider::{ProviderError, ProviderResult};

const BASE_URL: &str = "https://api.commandcode.ai";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub const CREDITS_PATH: &str = "/internal/billing/credits";
pub const SUMMARY_PATH: &str = "/internal/usage/summary";

#[async_trait]
pub trait CommandCodeClient: Send + Sync {
    async fn fetch_credits(&self, cookie: &str) -> ProviderResult<serde_json::Value>;
    async fn fetch_summary(&self, cookie: &str) -> ProviderResult<serde_json::Value>;
}

pub struct HttpCommandCodeClient {
    http: reqwest::Client,
}

impl HttpCommandCodeClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }
}

impl Default for HttpCommandCodeClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandCodeClient for HttpCommandCodeClient {
    async fn fetch_credits(&self, cookie: &str) -> ProviderResult<serde_json::Value> {
        self.get_json(&format!("{BASE_URL}{CREDITS_PATH}"), cookie)
            .await
    }

    async fn fetch_summary(&self, cookie: &str) -> ProviderResult<serde_json::Value> {
        self.get_json(&format!("{BASE_URL}{SUMMARY_PATH}"), cookie)
            .await
    }
}

impl HttpCommandCodeClient {
    async fn get_json(&self, url: &str, cookie: &str) -> ProviderResult<serde_json::Value> {
        let response = self
            .http
            .get(url)
            .header("Cookie", cookie)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let data = response
            .bytes()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        match status.as_u16() {
            200 => serde_json::from_slice(&data)
                .map_err(|error| ProviderError::Parse(error.to_string())),
            401 | 403 => Err(ProviderError::Auth(
                "commandcode session unauthorized or expired".into(),
            )),
            code => Err(ProviderError::Fetch(format!(
                "commandcode request failed: HTTP {code}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    /// Captured HTTP request: request line plus all headers.
    #[derive(Debug)]
    struct CapturedRequest {
        request_line: String,
        headers: String,
    }

    /// One-request test HTTP server: accepts a single connection, records the
    /// request, writes a canned response, then shuts down.
    fn serve_one(
        status_line: &str,
        body: &str,
    ) -> (String, std::thread::JoinHandle<CapturedRequest>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let status_line = status_line.to_string();
        let body = body.to_string();
        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("request line");
            let mut headers = String::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).expect("header line");
                if line == "\r\n" || line.is_empty() {
                    break;
                }
                headers.push_str(&line);
            }
            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let mut stream = stream;
            stream.write_all(response.as_bytes()).expect("respond");
            CapturedRequest {
                request_line: request_line.trim().to_string(),
                headers,
            }
        });
        (addr.to_string(), handle)
    }

    #[tokio::test]
    async fn get_json_maps_200_to_parsed_json() {
        let (addr, server) = serve_one("200 OK", r#"{"credits":123}"#);
        let client = HttpCommandCodeClient::new();
        let value = client
            .get_json(&format!("http://{addr}/x"), "cookie=1")
            .await
            .expect("200 must parse");
        assert_eq!(value["credits"], 123);
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn get_json_maps_401_to_auth_error() {
        let (addr, server) = serve_one("401 Unauthorized", "{}");
        let client = HttpCommandCodeClient::new();
        let result = client
            .get_json(&format!("http://{addr}/x"), "cookie=1")
            .await;
        assert!(
            matches!(result, Err(ProviderError::Auth(_))),
            "401 must map to Auth, got {result:?}"
        );
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn get_json_maps_403_to_auth_error() {
        let (addr, server) = serve_one("403 Forbidden", "{}");
        let client = HttpCommandCodeClient::new();
        let result = client
            .get_json(&format!("http://{addr}/x"), "cookie=1")
            .await;
        assert!(
            matches!(result, Err(ProviderError::Auth(_))),
            "403 must map to Auth, got {result:?}"
        );
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn get_json_maps_500_to_fetch_error() {
        let (addr, server) = serve_one("500 Internal Server Error", "{}");
        let client = HttpCommandCodeClient::new();
        let result = client
            .get_json(&format!("http://{addr}/x"), "cookie=1")
            .await;
        match &result {
            Err(ProviderError::Fetch(message)) => {
                assert!(
                    message.contains("HTTP 500"),
                    "must name the status: {message}"
                );
            }
            other => panic!("500 must map to Fetch, got {other:?}"),
        }
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn get_json_maps_invalid_200_body_to_parse_error() {
        let (addr, server) = serve_one("200 OK", "not-json{");
        let client = HttpCommandCodeClient::new();
        let result = client
            .get_json(&format!("http://{addr}/x"), "cookie=1")
            .await;
        assert!(
            matches!(result, Err(ProviderError::Parse(_))),
            "invalid body must map to Parse, got {result:?}"
        );
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn request_carries_cookie_and_accept_headers() {
        let (addr, server) = serve_one("200 OK", "{}");
        let client = HttpCommandCodeClient::new();
        let _ = client
            .get_json(
                &format!("http://{addr}/internal/usage/summary"),
                "session=abc",
            )
            .await;
        let captured = server.join().expect("server thread");
        assert!(
            captured
                .headers
                .lines()
                .any(|l| l.to_ascii_lowercase().starts_with("cookie:") && l.contains("session=abc")),
            "cookie header must be sent, got: {}",
            captured.headers
        );
        assert!(
            captured
                .headers
                .lines()
                .any(|l| l.to_ascii_lowercase().starts_with("accept:")
                    && l.contains("application/json")),
            "accept header must be sent, got: {}",
            captured.headers
        );
        assert!(
            captured
                .request_line
                .contains("GET /internal/usage/summary"),
            "must hit the requested path, got: {}",
            captured.request_line
        );
    }

    #[test]
    fn endpoints_match_fixed_har_capture() {
        assert_eq!(
            format!("{BASE_URL}{CREDITS_PATH}"),
            "https://api.commandcode.ai/internal/billing/credits"
        );
        assert_eq!(
            format!("{BASE_URL}{SUMMARY_PATH}"),
            "https://api.commandcode.ai/internal/usage/summary"
        );
        assert_eq!(REQUEST_TIMEOUT, Duration::from_secs(30));
    }
}
