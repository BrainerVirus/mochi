use std::collections::HashMap;
use std::sync::Mutex;

use super::log::log_line;
use super::{FrontendBootPayload, FrontendErrorPayload};
use crate::frontend::APP_SHELL_ASSET;
use tauri::Manager;

const MAX_EVENTS: usize = 300;

#[derive(Debug, Clone)]
pub struct DiagnosticEvent {
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct DiagnosticsInner {
    pub events: Vec<DiagnosticEvent>,
    pub window_boots: HashMap<String, FrontendBootPayload>,
}

pub struct DiagnosticsState {
    inner: Mutex<DiagnosticsInner>,
}

impl Default for DiagnosticsState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(DiagnosticsInner::default()),
        }
    }

    pub fn inner(&self) -> Result<std::sync::MutexGuard<'_, DiagnosticsInner>, String> {
        self.inner
            .lock()
            .map_err(|error| format!("diagnostics state lock poisoned: {error}"))
    }

    pub fn record_boot(&self, payload: FrontendBootPayload) {
        if let Ok(mut inner) = self.inner() {
            log_line(
                "frontend.boot",
                &format!(
                    "label={} href={} path={} target={} tauri={}",
                    payload.window_label,
                    payload.location_href,
                    payload.pathname,
                    payload.target_route,
                    payload.has_tauri_internals
                ),
            );
            inner
                .window_boots
                .insert(payload.window_label.clone(), payload.clone());
            push_event(
                &mut inner.events,
                "frontend.boot",
                format!(
                    "{} -> {} (tauri={})",
                    payload.window_label, payload.target_route, payload.has_tauri_internals
                ),
            );
        }
    }

    pub fn record_frontend_error(&self, payload: FrontendErrorPayload) {
        if let Ok(mut inner) = self.inner() {
            let label = payload
                .window_label
                .clone()
                .unwrap_or_else(|| "unknown".into());
            log_line(
                "frontend.error",
                &format!("label={label} message={}", payload.message),
            );
            push_event(
                &mut inner.events,
                "frontend.error",
                format!("{label}: {}", payload.message),
            );
        }
    }

    pub fn record_window_created(
        &self,
        label: &str,
        url: &str,
        decorations: bool,
        transparent: bool,
    ) {
        if let Ok(mut inner) = self.inner() {
            let detail = format!(
                "created label={label} url={url} shell={APP_SHELL_ASSET} decorations={decorations} transparent={transparent}"
            );
            log_line("window", &detail);
            push_event(&mut inner.events, "window.created", detail);
        }
    }

    pub fn record_window_event(&self, label: &str, event: &str) {
        if let Ok(mut inner) = self.inner() {
            let detail = format!("{label}: {event}");
            log_line("window.event", &detail);
            push_event(&mut inner.events, "window.event", detail);
        }
    }
}

fn push_event(events: &mut Vec<DiagnosticEvent>, kind: &str, detail: String) {
    events.push(DiagnosticEvent {
        kind: kind.into(),
        detail,
    });
    if events.len() > MAX_EVENTS {
        let overflow = events.len() - MAX_EVENTS;
        events.drain(0..overflow);
    }
}

pub fn log_visible_windows(app: &tauri::AppHandle) {
    for (label, window) in app.webview_windows() {
        let url = window
            .url()
            .map(|parsed| parsed.to_string())
            .unwrap_or_else(|error| format!("url-error:{error}"));
        let visible = window
            .is_visible()
            .map(|value| value.to_string())
            .unwrap_or_else(|error| format!("visible-error:{error}"));
        log_line(
            "window.snapshot",
            &format!("{label} url={url} visible={visible}"),
        );
    }
}
