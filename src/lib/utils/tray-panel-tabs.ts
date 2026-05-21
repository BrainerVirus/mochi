import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { getProviderLabel } from "./provider-labels";

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

export function buildTrayPanelTabs(snapshots: UsageSnapshot[]): TrayPanelTab[] {
  const usedPercents = snapshots.map((snapshot) => snapshot.primary.used_percent);
  const remainingPercents = snapshots.map((snapshot) => snapshot.primary.remaining_percent);

  const overviewTab: TrayPanelTab = {
    id: "overview",
    label: "Overview",
    usedPercent: usedPercents.length ? Math.max(...usedPercents) : 0,
    remainingPercent: remainingPercents.length ? Math.min(...remainingPercents) : 100,
  };

  const providerTabs = snapshots.map((snapshot) => ({
    id: snapshot.provider,
    label: getProviderLabel(snapshot.provider),
    usedPercent: snapshot.primary.used_percent,
    remainingPercent: snapshot.primary.remaining_percent,
  }));

  return [overviewTab, ...providerTabs];
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
