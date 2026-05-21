import { describe, expect, it } from "vitest";

import { formatUpdatedAgo } from "./format-updated-ago";

describe("formatUpdatedAgo", () => {
  const now = new Date(2026, 4, 21, 12, 0, 0);

  it("returns just now for recent updates", () => {
    const updatedAt = new Date(2026, 4, 21, 11, 59, 30).toISOString();
    expect(formatUpdatedAgo(updatedAt, now)).toBe("Updated just now");
  });

  it("returns minutes ago under one hour", () => {
    const updatedAt = new Date(2026, 4, 21, 11, 15, 0).toISOString();
    expect(formatUpdatedAgo(updatedAt, now)).toBe("Updated 45m ago");
  });

  it("returns hours ago under one day", () => {
    const updatedAt = new Date(2026, 4, 21, 8, 0, 0).toISOString();
    expect(formatUpdatedAgo(updatedAt, now)).toBe("Updated 4h ago");
  });
});
