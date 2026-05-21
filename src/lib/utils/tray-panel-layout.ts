/** Matches `src-tauri/tauri.conf.json` main window width. */
export const TRAY_PANEL_WIDTH_PX = 360;

/** Matches `src-tauri/tauri.conf.json` main window height. */
export const TRAY_PANEL_MAX_HEIGHT_PX = 480;

export function trayPanelShellClassName(): string {
  return "tray-panel bg-background text-foreground flex h-full max-h-full min-h-0 w-full flex-1 flex-col overflow-hidden rounded-mochi shadow-sm ring-1 ring-border";
}

export function trayPanelScrollRegionClassName(): string {
  return "min-h-0 flex-1";
}
