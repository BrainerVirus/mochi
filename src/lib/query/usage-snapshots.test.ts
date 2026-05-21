import { describe, expect, it } from "vitest";

import { queryKeys } from "./keys";
import { usageSnapshotsQueryOptions } from "./usage-snapshots";

describe("usageSnapshotsQueryOptions", () => {
  it("uses the centralized usage snapshots query key", () => {
    expect(usageSnapshotsQueryOptions.queryKey).toEqual(queryKeys.usageSnapshots);
  });
});
