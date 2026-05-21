import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { filterConfiguredSnapshots } from "./is-provider-configured";
import { getProviderLabel } from "./provider-labels";

function isEnabledProvider(provider: ProviderId, enabledProviders: ProviderId[]): boolean {
  return enabledProviders.includes(provider);
}

export interface TrayPanelTab {
  id: "overview" | ProviderId;
  label: string;
}

export function buildTrayPanelTabs(
  snapshots: UsageSnapshot[],
  enabledProviders: ProviderId[] = [],
): TrayPanelTab[] {
  const configuredSnapshots = filterConfiguredSnapshots(
    enabledProviders.length === 0
      ? snapshots
      : snapshots.filter((snapshot) => isEnabledProvider(snapshot.provider, enabledProviders)),
  );

  const overviewTab: TrayPanelTab = {
    id: "overview",
    label: "Overview",
  };

  const providerTabs = configuredSnapshots.map((snapshot) => ({
    id: snapshot.provider,
    label: getProviderLabel(snapshot.provider),
  }));

  return [overviewTab, ...providerTabs];
}

export function filterSnapshotsForTrayPanel(
  snapshots: UsageSnapshot[],
  enabledProviders: ProviderId[],
): UsageSnapshot[] {
  const enabledSnapshots =
    enabledProviders.length === 0
      ? snapshots
      : snapshots.filter((snapshot) => isEnabledProvider(snapshot.provider, enabledProviders));

  return filterConfiguredSnapshots(enabledSnapshots);
}

/** @deprecated Use filterSnapshotsForTrayPanel — kept for callers that only filter by enabled. */
export function filterSnapshotsForEnabledProviders(
  snapshots: UsageSnapshot[],
  enabledProviders: ProviderId[],
): UsageSnapshot[] {
  if (enabledProviders.length === 0) {
    return snapshots;
  }

  return snapshots.filter((snapshot) => isEnabledProvider(snapshot.provider, enabledProviders));
}
