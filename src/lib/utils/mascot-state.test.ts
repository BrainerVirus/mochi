import { describe, expect, it } from "vitest";

import type { UsageSnapshot } from "@/lib/schemas/usage";

import { getMascotStateFromSnapshots } from "./mascot-state";

function snapshot(usedPercent: number): UsageSnapshot {
  return {
    provider: "codex",
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

describe("getMascotStateFromSnapshots", () => {
  it("returns warning when usage fetch failed", () => {
    expect(getMascotStateFromSnapshots([], { isError: true })).toBe("warning");
  });

  it("returns critical when any provider is at or above 85%", () => {
    expect(getMascotStateFromSnapshots([snapshot(85)])).toBe("critical");
  });

  it("returns warning between 60% and 84%", () => {
    expect(getMascotStateFromSnapshots([snapshot(72)])).toBe("warning");
  });

  it("returns all-good when usage stays low", () => {
    expect(getMascotStateFromSnapshots([snapshot(12)])).toBe("all-good");
  });
});
