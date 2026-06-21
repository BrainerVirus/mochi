import { describe, expect, it } from "vitest";
import { validatePublishedFeeds } from "./validate-published-feeds.mjs";

describe("validatePublishedFeeds", () => {
  it("rejects empty path lists", async () => {
    await expect(
      validatePublishedFeeds({ baseUrl: "https://example.com", paths: [] }),
    ).rejects.toThrow(/at least one feed path/);
  });
});
