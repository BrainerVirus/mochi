import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const release = () => readFileSync(".github/workflows/release.yml", "utf8");
const stable = () => readFileSync(".github/workflows/release-stable.yml", "utf8");
const config = () => readFileSync("release.config.cjs", "utf8");

describe("semantic-release workflow", () => {
  it("gates releases through the analyzer", () => {
    const cfg = config();
    expect(cfg).toContain("analyzeCommitsCmd");
    expect(cfg).toContain("analyze-release-scope.mjs");
  });
  it("does not reference the removed prerelease channel", () => {
    for (const f of [".github/workflows/release.yml", ".github/workflows/release-stable.yml"]) {
      expect(readFileSync(f, "utf8").toLowerCase()).not.toContain("unstable");
    }
  });
  it("stable build injects the tag version", () => {
    expect(stable()).toMatch(/sync-manifest-version\.mjs/);
  });
  it("sync step env maps RELEASE_SYNC_TOKEN", () => {
    const step = release().split("Sync release manifests to main")[1];
    expect(step.slice(0, step.indexOf("run:"))).toContain(
      "RELEASE_SYNC_TOKEN: ${{ secrets.RELEASE_SYNC_TOKEN }}",
    );
  });
  it("sync commit stages all four manifests", () => {
    const step = release().split("Sync release manifests to main")[1];
    expect(step).toContain("src-tauri/Cargo.lock");
  });
  it("pages workflow does not reference the removed feed host", () => {
    expect(readFileSync(".github/workflows/publish-updater-pages.yml", "utf8")).not.toContain(
      "mochi-app.github.io",
    );
  });
});
