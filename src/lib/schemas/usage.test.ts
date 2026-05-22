import { describe, expect, it } from "vitest";

import { parseUsageSnapshots } from "./usage";

describe("parseUsageSnapshots health metadata", () => {
  it("accepts snapshots with health and stale metadata", () => {
    const snapshots = parseUsageSnapshots([
      {
        provider: "codex",
        primary: {
          label: "Session",
          used_percent: 80,
          remaining_percent: 20,
          resets_at: null,
        },
        secondary: null,
        updated_at: "2026-05-20T12:00:00Z",
        source: "codex-cli",
        health: "stale",
        is_stale: true,
        error: "provider fetch failed: network",
        last_fetch_attempt: {
          strategy_id: "live-fetch",
          succeeded: false,
          error: "provider fetch failed: network",
          attempted_at: "2026-05-20T12:01:00Z",
        },
      },
    ]);

    expect(snapshots[0]?.health).toBe("stale");
    expect(snapshots[0]?.is_stale).toBe(true);
    expect(snapshots[0]?.error).toContain("network");
  });

  it("defaults health fields when omitted", () => {
    const snapshots = parseUsageSnapshots([
      {
        provider: "cursor",
        primary: {
          label: "Session",
          used_percent: 0,
          remaining_percent: 100,
          resets_at: null,
        },
        secondary: null,
        updated_at: "2026-05-20T12:00:00Z",
        source: "Cursor",
      },
    ]);

    expect(snapshots[0]?.health).toBe("ok");
    expect(snapshots[0]?.is_stale).toBe(false);
  });
});

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
    expect(snapshots[0]?.health).toBe("ok");
    expect(snapshots[0]?.is_stale).toBe(false);
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
