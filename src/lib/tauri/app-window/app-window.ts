import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export const APP_WINDOW_LABEL = "settings";

/** Whether the current webview is a dedicated settings/about window. */
export function readIsAppWindow(): boolean {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return false;
  }

  try {
    return getCurrentWebviewWindow().label === APP_WINDOW_LABEL;
  } catch {
    return false;
  }
}
