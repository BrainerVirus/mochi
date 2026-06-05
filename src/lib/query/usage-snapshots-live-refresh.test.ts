import { describe, expect, it } from "vitest";

import { createUsageSnapshotsQueryOptions } from "./usage-snapshots";

describe("createUsageSnapshotsQueryOptions live refresh", () => {
  it("uses cached snapshots before settings are loaded", () => {
    const options = createUsageSnapshotsQueryOptions();

    expect(options.queryFn?.name).toBe("getUsageStates");
  });

  it("keeps scheduled polling on the cache read path", () => {
    const options = createUsageSnapshotsQueryOptions(300);

    expect(options.queryFn?.name).toBe("getUsageStates");
    expect(options.refetchInterval).toBe(300_000);
    expect(options.refetchIntervalInBackground).toBe(true);
  });
});
