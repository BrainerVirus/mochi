import { describe, expect, it } from "vitest";

import { usageRefreshIntervalMs } from "./usage-refetch-interval";

describe("usageRefreshIntervalMs", () => {
  it("converts refresh interval seconds to milliseconds", () => {
    expect(usageRefreshIntervalMs(300)).toBe(300_000);
  });

  it("supports the minimum allowed refresh interval", () => {
    expect(usageRefreshIntervalMs(30)).toBe(30_000);
  });
});
