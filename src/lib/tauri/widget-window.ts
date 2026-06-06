import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export const WIDGET_WINDOW_LABEL = "widget";

export function readIsWidgetWindow(): boolean {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return false;
  }

  try {
    return getCurrentWebviewWindow().label === WIDGET_WINDOW_LABEL;
  } catch {
    return false;
  }
}
