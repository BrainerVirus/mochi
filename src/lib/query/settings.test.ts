import { describe, expect, it } from "vitest";

import { queryKeys } from "./keys";
import { DEFAULT_MOCHI_SETTINGS } from "@/lib/schemas/settings";

import { settingsQueryOptions } from "./settings";

describe("settingsQueryOptions", () => {
  it("uses the centralized settings query key", () => {
    expect(settingsQueryOptions.queryKey).toEqual(queryKeys.settings);
  });

  it("uses backend defaults as placeholder data for a stable first render", () => {
    expect(settingsQueryOptions.placeholderData).toEqual(DEFAULT_MOCHI_SETTINGS);
  });
});
