/** Fixed row height shared with ScrollFadeRegion chevron overlay math. */
export const TRAY_SEGMENT_ROW_HEIGHT = "h-11" as const;

/** Tray page-tab strip — scrollable segments with fixed 1.5rem pill radii. */
export const TRAY_PAGE_TAB_DEFAULTS = {
  variant: "page-tabs" as const,
  rowHeight: TRAY_SEGMENT_ROW_HEIGHT,
  stretchItems: false,
  layout: "tray" as const,
};

/** Settings page-tab strip — full-width equal segments with .app-window --radius rounding. */
export const SETTINGS_PAGE_TAB_DEFAULTS = {
  variant: "page-tabs" as const,
  rowHeight: "h-9" as const,
  stretchItems: true,
  layout: "settings" as const,
};
