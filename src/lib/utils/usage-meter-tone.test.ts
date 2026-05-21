import { describe, expect, it } from "vitest";

import { getUsageMeterTone } from "./usage-meter-tone";

describe("getUsageMeterTone", () => {
  it("returns normal below 60%", () => {
    expect(getUsageMeterTone(0)).toBe("normal");
    expect(getUsageMeterTone(59)).toBe("normal");
  });

  it("returns warning from 60% up to 84%", () => {
    expect(getUsageMeterTone(60)).toBe("warning");
    expect(getUsageMeterTone(84)).toBe("warning");
  });

  it("returns critical at 85% and above", () => {
    expect(getUsageMeterTone(85)).toBe("critical");
    expect(getUsageMeterTone(100)).toBe("critical");
  });

  it("clamps out-of-range values", () => {
    expect(getUsageMeterTone(-10)).toBe("normal");
    expect(getUsageMeterTone(150)).toBe("critical");
  });
});
