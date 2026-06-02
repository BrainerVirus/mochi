import { describe, expect, it } from "vitest";

import { createUsageSnapshotsQueryOptions } from "./usage-snapshots";

describe("createUsageSnapshotsQueryOptions live refresh", () => {
  it("uses cached snapshots before settings are loaded", () => {
    const options = createUsageSnapshotsQueryOptions();

    expect(options.queryFn?.name).toBe("getUsageSnapshots");
  });

  it("refreshes enabled providers on the configured interval", () => {
    const options = createUsageSnapshotsQueryOptions(300);

    expect(options.queryFn?.name).toBe("refreshEnabledProviders");
    expect(options.refetchInterval).toBe(300_000);
    expect(options.refetchIntervalInBackground).toBe(true);
  });
});
