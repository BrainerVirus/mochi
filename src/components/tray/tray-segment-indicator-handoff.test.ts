import { describe, expect, it } from "vitest";

import { resolveHoverHandoffStart } from "@/components/tray/tray-segment-indicator";

describe("resolveHoverHandoffStart", () => {
  it("returns null when the hovered tab does not match the selection", () => {
    expect(
      resolveHoverHandoffStart({
        hoveredId: "alpha",
        targetTabId: "beta",
        hoverVisible: true,
        hoverMetrics: { x: 12, width: 64 },
      }),
    ).toBeNull();
  });

  it("returns hover metrics when clicking the hovered tab mid-tween", () => {
    expect(
      resolveHoverHandoffStart({
        hoveredId: "alpha",
        targetTabId: "alpha",
        hoverVisible: true,
        hoverMetrics: { x: 55, width: 66 },
      }),
    ).toEqual({ x: 55, width: 66 });
  });

  it("returns positioned hover metrics even before the fade-in has advanced", () => {
    expect(
      resolveHoverHandoffStart({
        hoveredId: "alpha",
        targetTabId: "alpha",
        hoverVisible: false,
        hoverMetrics: { x: 55, width: 66 },
      }),
    ).toEqual({ x: 55, width: 66 });
  });

  it("ignores an unplaced hover pill before handing off to active", () => {
    expect(
      resolveHoverHandoffStart({
        hoveredId: "alpha",
        targetTabId: "alpha",
        hoverVisible: false,
        hoverMetrics: { x: 0, width: 0 },
      }),
    ).toBeNull();
  });
});
