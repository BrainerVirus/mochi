import { describe, expect, it } from "vitest";

import { shouldRunProviderRefreshForTrayEvent } from "./use-tray-events";

describe("tray event refresh policy", () => {
  it("runs a real provider refresh before resyncing tray usage", () => {
    expect(shouldRunProviderRefreshForTrayEvent("tray-refresh")).toBe(true);
  });
});
