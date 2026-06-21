import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { validateFeedFile } from "./validate-updater-feed.mjs";

describe("validateFeedFile", () => {
  it("accepts a well-formed updater feed", async () => {
    const dir = mkdtempSync(path.join(tmpdir(), "mochi-feed-"));
    const feedPath = path.join(dir, "stable.json");
    writeFileSync(
      feedPath,
      `${JSON.stringify({
        version: "0.2.4",
        pub_date: "2026-06-06T12:00:00.000Z",
        platforms: {
          "darwin-aarch64": {
            url: "https://example.com/Mochi.app.tar.gz",
            signature: "sig",
          },
        },
      })}\n`,
    );

    await expect(validateFeedFile(feedPath)).resolves.toBe(true);
  });

  it("rejects feeds missing platform artifacts", async () => {
    const dir = mkdtempSync(path.join(tmpdir(), "mochi-feed-"));
    const feedPath = path.join(dir, "broken.json");
    writeFileSync(
      feedPath,
      `${JSON.stringify({
        version: "0.2.4",
        pub_date: "2026-06-06T12:00:00.000Z",
        platforms: {
          "darwin-aarch64": { url: "https://example.com/app.tar.gz" },
        },
      })}\n`,
    );

    await expect(validateFeedFile(feedPath)).rejects.toThrow(/invalid updater artifact/);
  });
});
