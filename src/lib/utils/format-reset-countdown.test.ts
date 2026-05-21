import { describe, expect, it } from "vitest";

import { formatResetCountdown } from "./format-reset-countdown";

describe("formatResetCountdown", () => {
  const now = new Date("2026-05-21T12:00:00Z");

  it("returns null when reset time is missing", () => {
    expect(formatResetCountdown(null, now)).toBeNull();
  });

  it("returns minutes when reset is under one hour away", () => {
    expect(formatResetCountdown("2026-05-21T12:45:00Z", now)).toBe("45m");
  });

  it("returns hours and minutes when reset is under one day away", () => {
    expect(formatResetCountdown("2026-05-21T15:30:00Z", now)).toBe("3h 30m");
  });

  it("returns days and hours when reset is more than a day away", () => {
    expect(formatResetCountdown("2026-05-23T18:00:00Z", now)).toBe("2d 6h");
  });

  it("returns resetting copy when reset time has passed", () => {
    expect(formatResetCountdown("2026-05-21T11:00:00Z", now)).toBe("Resetting…");
  });
});
