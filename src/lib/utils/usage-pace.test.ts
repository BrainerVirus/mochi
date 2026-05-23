import { describe, expect, it } from "vitest";

import type { UsageWindow } from "@/lib/schemas/usage";

import { reserveDetailLeft, usagePaceDetail } from "./usage-pace";

function window(overrides: Partial<UsageWindow>): UsageWindow {
  return {
    label: "Weekly",
    used_percent: 10,
    remaining_percent: 90,
    resets_at: null,
    ...overrides,
  };
}

describe("usagePaceDetail", () => {
  it("returns on-pace label when usage matches elapsed window", () => {
    const now = new Date("2026-05-22T12:00:00Z");
    const resetsAt = "2026-05-29T12:00:00Z";
    const detail = usagePaceDetail(
      window({ used_percent: 0, remaining_percent: 100, resets_at: resetsAt }),
      now,
    );

    expect(detail?.leftLabel).toBe("On pace");
    expect(detail?.rightLabel).toBeNull();
  });

  it("falls back to reserve percent without reset time", () => {
    expect(reserveDetailLeft(window({ remaining_percent: 42, resets_at: null }))).toBe(
      "42% in reserve",
    );
  });
});
