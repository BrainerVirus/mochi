import type { UsageSnapshot } from "@/lib/schemas/usage";

/** Placeholder timestamp returned by unconfigured static provider snapshots. */
export const STATIC_SNAPSHOT_EPOCH = "1970-01-01T00:00:00Z";

/**
 * A provider is "configured" when it has real usage data from a successful fetch,
 * not a static placeholder snapshot (0% windows, epoch updated_at).
 */
export function isProviderConfigured(snapshot: UsageSnapshot): boolean {
  return snapshot.updated_at !== STATIC_SNAPSHOT_EPOCH;
}

export function filterConfiguredSnapshots(snapshots: UsageSnapshot[]): UsageSnapshot[] {
  return snapshots.filter(isProviderConfigured);
}
