import { describe, expect, it } from "vitest";

import {
  clearTraySegmentHover,
  clearTraySegmentPointerHover,
  createTraySegmentHoverState,
  getTraySegmentHoverTarget,
  setTraySegmentPointerHover,
} from "@/components/tray/tray-segment-hover-state";

describe("tray segment hover state", () => {
  it("uses the pointer target instead of a stale focused tab", () => {
    const state = createTraySegmentHoverState();

    state.focusedId = "overview";
    setTraySegmentPointerHover(state, "codex");

    expect(getTraySegmentHoverTarget(state)).toBe("codex");
  });

  it("ignores a late leave from a previous tab after another tab is hovered", () => {
    const state = createTraySegmentHoverState();

    setTraySegmentPointerHover(state, "overview");
    setTraySegmentPointerHover(state, "codex");
    clearTraySegmentPointerHover(state, "overview");

    expect(getTraySegmentHoverTarget(state)).toBe("codex");
  });

  it("does not show the hover pill from focus alone", () => {
    const state = createTraySegmentHoverState();

    state.focusedId = "overview";

    expect(getTraySegmentHoverTarget(state)).toBeNull();
  });

  it("clears stale hover when selection changes outside the tab item", () => {
    const state = createTraySegmentHoverState();

    setTraySegmentPointerHover(state, "overview");
    clearTraySegmentHover(state);

    expect(getTraySegmentHoverTarget(state)).toBeNull();
  });
});
