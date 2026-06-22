import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

import {
  initialRouteForWindowLabel,
  shouldNavigateFromPackagedShell,
} from "@/lib/tauri/initial-window-route";

export type FrontendBootPayload = {
  windowLabel: string;
  locationHref: string;
  pathname: string;
  userAgent: string;
  hasTauriInternals: boolean;
  targetRoute: string;
  tauriLabel: string | null;
  tauriLabelError: string | null;
};

export type FrontendErrorPayload = {
  windowLabel: string | null;
  message: string;
  source: string | null;
  stack: string | null;
};

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function collectBootPayload(): FrontendBootPayload | null {
  if (!isTauriRuntime()) {
    return null;
  }

  let windowLabel = "unknown";
  let tauriLabel: string | null = null;
  let tauriLabelError: string | null = null;

  try {
    tauriLabel = getCurrentWebviewWindow().label;
    windowLabel = tauriLabel;
  } catch (error) {
    tauriLabelError = error instanceof Error ? error.message : String(error);
  }

  const pathname = window.location.pathname;
  const targetRoute = shouldNavigateFromPackagedShell(pathname)
    ? initialRouteForWindowLabel(windowLabel)
    : pathname;

  return {
    windowLabel,
    locationHref: window.location.href,
    pathname,
    userAgent: navigator.userAgent,
    hasTauriInternals: true,
    targetRoute,
    tauriLabel,
    tauriLabelError,
  };
}

export async function reportFrontendBoot(): Promise<void> {
  const payload = collectBootPayload();
  if (!payload) {
    return;
  }

  try {
    await invoke("report_frontend_boot", { payload });
  } catch {
    // Diagnostics must not break the UI if IPC is unavailable.
  }
}

export function logFrontendDebug(scope: string, message: string): void {
  if (import.meta.env.DEV) {
    // ponytail: dev-only trace; production relies on reportFrontendError debug sources
    // oxlint-disable-next-line no-console -- intentional dev-only diagnostics trace
    console.debug(`[${scope}] ${message}`);
  }
}

export async function reportFrontendError(
  message: string,
  source?: string,
  stack?: string,
): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  let windowLabel: string | null = null;
  try {
    windowLabel = getCurrentWebviewWindow().label;
  } catch {
    windowLabel = null;
  }

  const payload: FrontendErrorPayload = {
    windowLabel,
    message,
    source: source ?? null,
    stack: stack ?? null,
  };

  try {
    await invoke("report_frontend_error", { payload });
  } catch {
    // Ignore — best-effort diagnostics only.
  }
}

export function installDiagnosticsHandlers(): void {
  if (!isTauriRuntime() || typeof window === "undefined") {
    return;
  }

  window.addEventListener("error", (event) => {
    void reportFrontendError(
      event.message || "window error",
      event.filename || undefined,
      event.error instanceof Error ? event.error.stack : undefined,
    );
  });

  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason;
    const message =
      reason instanceof Error
        ? reason.message
        : typeof reason === "string"
          ? reason
          : "unhandled promise rejection";
    const stack = reason instanceof Error ? reason.stack : undefined;
    void reportFrontendError(message, "unhandledrejection", stack);
  });
}
