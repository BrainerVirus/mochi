import { describe, expect, it } from "vitest";

import {
  usageMeterEmptyFillTransform,
  formatUsageMeterLeftLabel,
  resolveUsageMeterDisplayPercent,
  resolveUsageMeterFillStartPercent,
  USAGE_METER_FILL_DURATION_S,
  USAGE_METER_FILL_EASE,
  usageMeterFillTranslateX,
} from "./usage-meter-fill-animation";

describe("usageMeterFillAnimation", () => {
  it("maps used percent to indicator translateX", () => {
    expect(usageMeterFillTranslateX(0)).toBe("-100%");
    expect(usageMeterFillTranslateX(50)).toBe("-50%");
    expect(usageMeterFillTranslateX(100)).toBe("0%");
  });

  it("starts animated indicators visually empty before GSAP runs", () => {
    expect(usageMeterEmptyFillTransform()).toBe("translateX(-100%)");
  });

  it("formats the displayed left label", () => {
    expect(formatUsageMeterLeftLabel(90.4)).toBe("90% left");
    expect(formatUsageMeterLeftLabel(90.5)).toBe("91% left");
    expect(formatUsageMeterLeftLabel(-10)).toBe("0% left");
    expect(formatUsageMeterLeftLabel(140)).toBe("100% left");
  });

  it("uses remaining percent as the displayed fill value", () => {
    expect(resolveUsageMeterDisplayPercent(10, 90)).toBe(90);
    expect(resolveUsageMeterDisplayPercent(10)).toBe(90);
    expect(resolveUsageMeterDisplayPercent(95, 5)).toBe(5);
  });

  it("clamps displayed fill values", () => {
    expect(resolveUsageMeterDisplayPercent(-10)).toBe(100);
    expect(resolveUsageMeterDisplayPercent(125)).toBe(0);
    expect(resolveUsageMeterDisplayPercent(10, 140)).toBe(100);
    expect(resolveUsageMeterDisplayPercent(10, -20)).toBe(0);
  });

  it("uses a subtle fill tween duration and easing", () => {
    expect(USAGE_METER_FILL_DURATION_S).toBeGreaterThanOrEqual(0.4);
    expect(USAGE_METER_FILL_DURATION_S).toBeLessThanOrEqual(0.6);
    expect(USAGE_METER_FILL_EASE).toBe("power2.out");
  });

  it("restarts fill from empty when tab activation changes", () => {
    expect(resolveUsageMeterFillStartPercent(42, "codex", "overview")).toBe(0);
    expect(resolveUsageMeterFillStartPercent(42, "codex:2", "codex:1")).toBe(0);
    expect(resolveUsageMeterFillStartPercent(42, "codex", "codex")).toBe(42);
    expect(resolveUsageMeterFillStartPercent(null, "codex", "codex")).toBe(0);
  });
});
