import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { getProviderLabel } from "./provider-labels";

function isEnabledProvider(provider: ProviderId, enabledProviders: ProviderId[]): boolean {
  return enabledProviders.includes(provider);
}

export interface TrayPanelTab {
  id: "overview" | ProviderId;
  label: string;
  usedPercent: number;
  remainingPercent: number;
}

export interface OverviewMetrics {
  providerCount: number;
  highestUsedPercent: number;
  averageUsedPercent: number;
  healthyCount: number;
}

const HEALTHY_THRESHOLD = 60;

export function buildTrayPanelTabs(
  snapshots: UsageSnapshot[],
  enabledProviders: ProviderId[] = [],
): TrayPanelTab[] {
  const enabledSnapshots =
    enabledProviders.length === 0
      ? snapshots
      : snapshots.filter((snapshot) => isEnabledProvider(snapshot.provider, enabledProviders));

  const usedPercents = enabledSnapshots.map((snapshot) => snapshot.primary.used_percent);
  const remainingPercents = enabledSnapshots.map((snapshot) => snapshot.primary.remaining_percent);

  const overviewTab: TrayPanelTab = {
    id: "overview",
    label: "Overview",
    usedPercent: usedPercents.length ? Math.max(...usedPercents) : 0,
    remainingPercent: remainingPercents.length ? Math.min(...remainingPercents) : 100,
  };

  const providerTabs = enabledSnapshots.map((snapshot) => ({
    id: snapshot.provider,
    label: getProviderLabel(snapshot.provider),
    usedPercent: snapshot.primary.used_percent,
    remainingPercent: snapshot.primary.remaining_percent,
  }));

  return [overviewTab, ...providerTabs];
}

export function filterSnapshotsForEnabledProviders(
  snapshots: UsageSnapshot[],
  enabledProviders: ProviderId[],
): UsageSnapshot[] {
  if (enabledProviders.length === 0) {
    return snapshots;
  }

  return snapshots.filter((snapshot) => isEnabledProvider(snapshot.provider, enabledProviders));
}

export function getOverviewMetrics(snapshots: UsageSnapshot[]): OverviewMetrics {
  if (snapshots.length === 0) {
    return {
      providerCount: 0,
      highestUsedPercent: 0,
      averageUsedPercent: 0,
      healthyCount: 0,
    };
  }

  const usedPercents = snapshots.map((snapshot) => snapshot.primary.used_percent);
  const total = usedPercents.reduce((sum, value) => sum + value, 0);

  return {
    providerCount: snapshots.length,
    highestUsedPercent: Math.round(Math.max(...usedPercents)),
    averageUsedPercent: Math.round(total / snapshots.length),
    healthyCount: usedPercents.filter((value) => value < HEALTHY_THRESHOLD).length,
  };
}
