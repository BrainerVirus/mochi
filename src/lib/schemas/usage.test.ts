import { describe, expect, it } from "vitest";

import { parseUsageSnapshots } from "./usage";

describe("parseUsageSnapshots", () => {
  it("accepts a valid usage snapshot array", () => {
    const snapshots = parseUsageSnapshots([
      {
        provider: "claude",
        primary: {
          label: "Session",
          used_percent: 42,
          remaining_percent: 58,
          resets_at: null,
        },
        secondary: null,
        updated_at: "2026-05-20T12:00:00Z",
        source: "Claude",
      },
    ]);

    expect(snapshots).toHaveLength(1);
    expect(snapshots[0]?.provider).toBe("claude");
  });

  it("rejects snapshots with invalid provider ids", () => {
    expect(() =>
      parseUsageSnapshots([
        {
          provider: "unknown-provider",
          primary: {
            label: "Session",
            used_percent: 0,
            remaining_percent: 100,
            resets_at: null,
          },
          secondary: null,
          updated_at: "2026-05-20T12:00:00Z",
          source: "Unknown",
        },
      ]),
    ).toThrow(/Invalid option|invalid/i);
  });
});
