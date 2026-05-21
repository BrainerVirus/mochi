import { describe, expect, it } from "vitest";

import { queryKeys } from "./keys";
import { createUsageSnapshotsQueryOptions } from "./usage-snapshots";

describe("createUsageSnapshotsQueryOptions", () => {
  it("uses the centralized usage snapshots query key", () => {
    expect(createUsageSnapshotsQueryOptions().queryKey).toEqual(queryKeys.usageSnapshots);
  });

  it("polls usage snapshots on the configured refresh interval", () => {
    const options = createUsageSnapshotsQueryOptions(120);

    expect(options.refetchInterval).toBe(120_000);
    expect(options.refetchIntervalInBackground).toBe(true);
  });

  it("does not poll until a refresh interval is provided", () => {
    const options = createUsageSnapshotsQueryOptions();

    expect(options.refetchInterval).toBeUndefined();
  });
});
