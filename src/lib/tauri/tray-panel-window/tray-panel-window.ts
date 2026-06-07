import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export const TRAY_PANEL_WINDOW_LABEL = "main";

/** Whether the current webview is the tray popover (`main` window label). */
export function readIsTrayPanelWindow(): boolean {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return false;
  }

  try {
    return getCurrentWebviewWindow().label === TRAY_PANEL_WINDOW_LABEL;
  } catch {
    return false;
  }
}
