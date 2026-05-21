import { describe, expect, it } from "vitest";

import {
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
});
