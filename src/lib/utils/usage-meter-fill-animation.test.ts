import { describe, expect, it } from "vitest";

import {
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

  it("uses a subtle fill tween duration and easing", () => {
    expect(USAGE_METER_FILL_DURATION_S).toBeGreaterThanOrEqual(0.4);
    expect(USAGE_METER_FILL_DURATION_S).toBeLessThanOrEqual(0.6);
    expect(USAGE_METER_FILL_EASE).toBe("power2.out");
  });

  it("restarts fill from empty when tab activation changes", () => {
    expect(resolveUsageMeterFillStartPercent(42, "codex", "overview")).toBe(0);
    expect(resolveUsageMeterFillStartPercent(42, "codex", "codex")).toBe(42);
    expect(resolveUsageMeterFillStartPercent(null, "codex", "codex")).toBe(0);
  });
});
