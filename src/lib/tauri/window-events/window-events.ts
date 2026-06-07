import { readIsAppWindow } from "@/lib/tauri/app-window";
import { readIsTrayPanelWindow } from "@/lib/tauri/tray-panel-window";

export function shouldHandleTrayNavigateForWindow(
  isTrayPanelWindow: boolean,
  isAppWindow: boolean,
): boolean {
  return isTrayPanelWindow && !isAppWindow;
}

export function shouldHandleAppNavigateForWindow(
  isTrayPanelWindow: boolean,
  isAppWindow: boolean,
): boolean {
  return isAppWindow && !isTrayPanelWindow;
}

/** Tray popover navigation events must not reach settings/about webviews. */
export function shouldHandleTrayNavigateEvent(): boolean {
  return shouldHandleTrayNavigateForWindow(readIsTrayPanelWindow(), readIsAppWindow());
}

/** Settings/about navigation events must not reach the tray popover webview. */
export function shouldHandleAppNavigateEvent(): boolean {
  return shouldHandleAppNavigateForWindow(readIsTrayPanelWindow(), readIsAppWindow());
}
