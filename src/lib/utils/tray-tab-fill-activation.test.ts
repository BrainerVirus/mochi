import { describe, expect, it } from "vitest";

import {
  formatTabFillActivationKey,
  nextTabFillActivationState,
} from "./tray-tab-fill-activation";

describe("trayTabFillActivation", () => {
  it("increments generation on every tab switch, including returning to the same tab id", () => {
    let state = { tab: "overview", generation: 0 };
    expect(formatTabFillActivationKey(state)).toBe("overview:0");

    state = nextTabFillActivationState(state, "codex");
    expect(formatTabFillActivationKey(state)).toBe("codex:1");

    state = nextTabFillActivationState(state, "overview");
    expect(formatTabFillActivationKey(state)).toBe("overview:2");
  });
});
