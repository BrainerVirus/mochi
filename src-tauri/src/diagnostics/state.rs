use std::collections::HashMap;
use std::sync::Mutex;

use super::log::log_line;
use super::{FrontendBootPayload, FrontendErrorPayload};
use crate::frontend::APP_SHELL_ASSET;
use crate::linux_window_controls::LinuxWindowControlDiagnostics;
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

pub fn format_logical_size(size: Option<(f64, f64)>) -> String {
    size.map(|(width, height)| format!("{width:.0}x{height:.0}"))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn window_lifecycle_detail(
    label: &str,
    phase: &str,
    experiment: &str,
    creation: &str,
    initial_visibility: &str,
    outer_size: Option<(f64, f64)>,
    inner_size: Option<(f64, f64)>,
) -> String {
    format!(
        "label={label} phase={phase} experiment={experiment} creation={creation} initial_visibility={initial_visibility} outer={} inner={}",
        format_logical_size(outer_size),
        format_logical_size(inner_size)
    )
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

    pub fn record_linux_window_controls(&self, diagnostics: LinuxWindowControlDiagnostics) {
        if let Ok(mut inner) = self.inner() {
            let detail = format!(
                "label={} platform={} source={} decorations={} ok={} error={:?} resizable={} ok={} error={:?}",
                diagnostics.label,
                diagnostics.platform,
                diagnostics.creation_source,
                diagnostics.decorations_action,
                diagnostics.decorations_ok,
                diagnostics.decorations_error,
                diagnostics.resizable_action,
                diagnostics.resizable_ok,
                diagnostics.resizable_error
            );
            log_line("window.linux_controls", &detail);
            push_event(&mut inner.events, "window.linux_controls", detail);
        }
    }

    pub fn record_window_lifecycle(
        &self,
        label: &str,
        phase: &str,
        experiment: &str,
        creation: &str,
        initial_visibility: &str,
        outer_size: Option<(f64, f64)>,
        inner_size: Option<(f64, f64)>,
    ) {
        let detail = window_lifecycle_detail(
            label,
            phase,
            experiment,
            creation,
            initial_visibility,
            outer_size,
            inner_size,
        );
        if let Ok(mut inner) = self.inner() {
            log_line("window.lifecycle", &detail);
            push_event(&mut inner.events, "window.lifecycle", detail);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_lifecycle_detail_includes_experiment_and_sequence() {
        let detail = window_lifecycle_detail(
            "settings",
            "show",
            "baseline-sequenced-logs",
            "startup-precreate",
            "hidden",
            Some((520.0, 560.0)),
            Some((520.0, 560.0)),
        );

        assert!(detail.contains("label=settings"));
        assert!(detail.contains("phase=show"));
        assert!(detail.contains("experiment=baseline-sequenced-logs"));
        assert!(detail.contains("creation=startup-precreate"));
        assert!(detail.contains("initial_visibility=hidden"));
        assert!(detail.contains("outer=520x560"));
        assert!(detail.contains("inner=520x560"));
    }
}
