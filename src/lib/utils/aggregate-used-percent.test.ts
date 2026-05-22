import { describe, expect, it } from "vitest";

import type { UsageSnapshot } from "@/lib/schemas/usage";

import { aggregateUsedPercent } from "./aggregate-used-percent";

function snapshot(usedPercent: number): UsageSnapshot {
  return {
    provider: "claude",
    primary: {
      label: "Session",
      used_percent: usedPercent,
      remaining_percent: 100 - usedPercent,
      resets_at: null,
    },
    secondary: null,
    updated_at: "1970-01-01T00:00:00Z",
    source: "test",
    health: "ok",
    is_stale: false,
  };
}

describe("aggregateUsedPercent", () => {
  it("returns the max primary usage across snapshots", () => {
    expect(aggregateUsedPercent([snapshot(12), snapshot(67), snapshot(41)])).toBe(67);
  });

  it("returns zero when there are no snapshots", () => {
    expect(aggregateUsedPercent([])).toBe(0);
  });
});
