import { describe, expect, it } from "vitest";

import { usageSnapshotsEmptyMessage } from "./usage-snapshots-empty-message";

describe("usageSnapshotsEmptyMessage", () => {
  it("describes the all-disabled case", () => {
    expect(usageSnapshotsEmptyMessage(0)).toContain("All providers are disabled");
  });

  it("describes the default empty snapshots case", () => {
    expect(usageSnapshotsEmptyMessage(2)).toContain("No provider usage snapshots yet");
  });
});
