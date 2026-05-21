import { describe, expect, it } from "vitest";

import { queryKeys } from "./keys";
import { settingsQueryOptions } from "./settings";

describe("settingsQueryOptions", () => {
  it("uses the centralized settings query key", () => {
    expect(settingsQueryOptions.queryKey).toEqual(queryKeys.settings);
  });
});
