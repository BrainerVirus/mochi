import type { UsageSnapshot } from "@/lib/schemas/usage";

/** Max primary-window usage across enabled provider snapshots (matches Rust tray aggregation). */
export function aggregateUsedPercent(snapshots: UsageSnapshot[]): number {
  if (snapshots.length === 0) {
    return 0;
  }

  const max = Math.max(...snapshots.map((snapshot) => snapshot.primary.used_percent));
  return Math.min(100, Math.round(max));
}
