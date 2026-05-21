import { describe, expect, it } from "vitest";

import { formatResetLine } from "./format-reset-line";

describe("formatResetLine", () => {
  const now = new Date(2026, 4, 21, 12, 0, 0);

  it("returns null when reset time is missing", () => {
    expect(formatResetLine(null, now)).toBeNull();
  });

  it("returns Resets HH:MM for same-day resets", () => {
    const resetAt = new Date(2026, 4, 21, 18, 24, 0).toISOString();
    expect(formatResetLine(resetAt, now)).toBe("Resets 18:24");
  });

  it("returns Resets now when reset time has passed", () => {
    const resetAt = new Date(2026, 4, 21, 11, 0, 0).toISOString();
    expect(formatResetLine(resetAt, now)).toBe("Resets now");
  });

  it("returns tomorrow label for next-day resets", () => {
    const resetAt = new Date(2026, 4, 22, 9, 30, 0).toISOString();
    expect(formatResetLine(resetAt, now)).toBe("Resets tomorrow, 09:30");
  });
});
