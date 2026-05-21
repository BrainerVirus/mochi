/**
 * Tray panel vertical rhythm (CodexBar-inspired density on Tailwind spacing scale).
 *
 * Groups: Stats (header + meters) → Links (dashboard/status) → System (refresh/settings/quit).
 * Inter-group gaps are owned by {@link trayPanelDividerClassName} (`pt-2` + line + `mb-1`).
 */
export const trayPanelSpacing = {
  /** Horizontal padding for panel-level columns and inset dividers. */
  contentX: "px-3",
  /** Space below the tab strip before provider content. */
  contentTop: "pt-3",
  /** Space between provider header and the meter group. */
  headerToMeters: "mt-2",
  /** Space between session and weekly meters. */
  meterGap: "gap-3",
  /** Above a group divider line. */
  dividerBefore: "pt-2",
  /** Below a group divider line, before the next group's rows. */
  dividerAfter: "mb-1",
  /** Bottom padding for the system-actions footer block. */
  footerBottom: "pb-1",
} as const;

/** Shared divider wrapper: consistent pt/mb; optional horizontal inset at panel edges. */
export function trayPanelDividerClassName(inset = false): string {
  return inset
    ? `${trayPanelSpacing.contentX} ${trayPanelSpacing.dividerBefore} pb-0`
    : `${trayPanelSpacing.dividerBefore} pb-0`;
}
