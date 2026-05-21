/** Matches `src-tauri/tauri.conf.json` main window width. */
export const TRAY_PANEL_WIDTH_PX = 360;

/** Minimum popover height (header + empty state). */
export const TRAY_PANEL_MIN_HEIGHT_PX = 160;

/** Fallback max height when viewport size is unavailable. */
export const TRAY_PANEL_DEFAULT_MAX_HEIGHT_PX = 480;

/** Gap between panel top/bottom and screen edge (`max-h-[calc(100svh-Npx)]`). */
export const TRAY_PANEL_VIEWPORT_MARGIN_PX = 16;

/** @deprecated Use TRAY_PANEL_DEFAULT_MAX_HEIGHT_PX */
export const TRAY_PANEL_MAX_HEIGHT_PX = TRAY_PANEL_DEFAULT_MAX_HEIGHT_PX;

export function trayPanelMaxHeightPx(viewportHeight: number): number {
  return Math.max(TRAY_PANEL_MIN_HEIGHT_PX, viewportHeight - TRAY_PANEL_VIEWPORT_MARGIN_PX);
}

export function clampTrayPanelHeight(
  contentHeight: number,
  viewportHeight = TRAY_PANEL_DEFAULT_MAX_HEIGHT_PX + TRAY_PANEL_VIEWPORT_MARGIN_PX,
): number {
  const maxHeight = trayPanelMaxHeightPx(viewportHeight);
  return Math.min(maxHeight, Math.max(TRAY_PANEL_MIN_HEIGHT_PX, Math.ceil(contentHeight)));
}

type TrayPanelContentMeasure = Pick<HTMLElement, "scrollHeight">;
type TrayPanelFooterMeasure = Pick<HTMLElement, "offsetHeight">;

/** Natural scroll content height plus fixed footer (excluded from scroll measurement). */
export function measureTrayPanelLayoutHeight(
  contentEl: TrayPanelContentMeasure | null,
  footerEl: TrayPanelFooterMeasure | null,
): number {
  const contentHeight = contentEl?.scrollHeight ?? 0;
  const footerHeight = footerEl?.offsetHeight ?? 0;
  return contentHeight + footerHeight;
}

export function trayPanelShellClassName(): string {
  return "tray-panel bg-background text-foreground flex h-full max-h-[calc(100svh-16px)] min-h-0 w-full flex-1 flex-col overflow-hidden rounded-mochi shadow-sm ring-1 ring-border";
}

export function trayPanelScrollRegionClassName(): string {
  return "min-h-0 flex-1";
}
