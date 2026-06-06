import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

for (const workflow of [
  ".github/workflows/release-stable.yml",
  ".github/workflows/release-unstable.yml",
]) {
  describe(workflow, () => {
    const source = readFileSync(workflow, "utf8");

    it("requires updater signing secrets before building", () => {
      expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY");
      expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
      expect(source).toContain("MOCHI_UPDATER_PUBLIC_KEY");
    });

    it("builds and validates updater feeds", () => {
      expect(source).toContain("scripts/release/collect-updater-artifacts.mjs");
      expect(source).toContain("scripts/release/build-updater-feed.mjs");
      expect(source).toContain("scripts/release/validate-updater-feed.mjs");
      expect(source).toContain("https://mochi-app.github.io/mochi/updates");
      expect(source).toContain("curl --fail");
    });
  });
}
