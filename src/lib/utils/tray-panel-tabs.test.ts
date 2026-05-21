import { describe, expect, it } from "vitest";

import type { UsageSnapshot } from "@/lib/schemas/usage";

import { buildTrayPanelTabs, getOverviewMetrics } from "./tray-panel-tabs";

function snapshot(
  provider: UsageSnapshot["provider"],
  usedPercent: number,
  secondaryUsed?: number,
): UsageSnapshot {
  return {
    provider,
    primary: {
      label: "Session",
      used_percent: usedPercent,
      remaining_percent: 100 - usedPercent,
      resets_at: null,
    },
    secondary: secondaryUsed
      ? {
          label: "Weekly",
          used_percent: secondaryUsed,
          remaining_percent: 100 - secondaryUsed,
          resets_at: null,
        }
      : null,
    updated_at: "2026-05-21T12:00:00Z",
    source: provider,
  };
}

describe("buildTrayPanelTabs", () => {
  it("includes an overview tab plus one tab per provider snapshot", () => {
    const tabs = buildTrayPanelTabs([snapshot("codex", 40), snapshot("cursor", 72)]);

    expect(tabs.map((tab) => tab.id)).toEqual(["overview", "codex", "cursor"]);
    expect(tabs[0]?.label).toBe("Overview");
    expect(tabs[1]?.usedPercent).toBe(40);
    expect(tabs[2]?.usedPercent).toBe(72);
  });

  it("only includes enabled provider tabs when enabled list is provided", () => {
    const tabs = buildTrayPanelTabs(
      [snapshot("codex", 40), snapshot("cursor", 72), snapshot("claude", 10)],
      ["codex", "claude"],
    );

    expect(tabs.map((tab) => tab.id)).toEqual(["overview", "codex", "claude"]);
  });
});

describe("getOverviewMetrics", () => {
  it("summarizes provider usage for the overview grid", () => {
    const metrics = getOverviewMetrics([
      snapshot("codex", 40),
      snapshot("cursor", 72, 55),
      snapshot("claude", 18),
    ]);

    expect(metrics).toEqual({
      providerCount: 3,
      highestUsedPercent: 72,
      averageUsedPercent: 43,
      healthyCount: 2,
    });
  });

  it("returns zeros when there are no snapshots", () => {
    expect(getOverviewMetrics([])).toEqual({
      providerCount: 0,
      highestUsedPercent: 0,
      averageUsedPercent: 0,
      healthyCount: 0,
    });
  });
});
