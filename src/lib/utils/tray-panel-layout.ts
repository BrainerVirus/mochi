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

export const TRAY_PANEL_CONTENT_SELECTOR = "[data-tray-panel-content]";
export const TRAY_PANEL_SEPARATOR_SELECTOR = "[data-tray-panel-separator]";
export const TRAY_PANEL_FOOTER_SELECTOR = "[data-tray-panel-footer]";

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

export function sumTrayPanelBlockHeights({
  contentScrollHeight,
  separatorHeight,
  footerHeight,
}: {
  contentScrollHeight: number;
  separatorHeight: number;
  footerHeight: number;
}): number {
  return contentScrollHeight + separatorHeight + footerHeight;
}

/** Natural height of the unified tray column (tabs, content, divider, and menu options). */
export function measureTrayPanelLayoutHeight(layoutEl: HTMLElement | null): number {
  if (!layoutEl) {
    return 0;
  }

  const contentEl = layoutEl.querySelector<HTMLElement>(TRAY_PANEL_CONTENT_SELECTOR);

  return (contentEl as TrayPanelContentMeasure | null)?.scrollHeight ?? 0;
}

export function trayPanelShellClassName(): string {
  return "tray-panel bg-background text-foreground flex h-full max-h-[calc(100svh-16px)] min-h-0 w-full flex-1 flex-col overflow-hidden rounded-mochi pt-3 shadow-sm ring-1 ring-border";
}

export function trayPanelScrollRegionClassName(): string {
  return "min-h-0 flex-1";
}
