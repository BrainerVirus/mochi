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
  it("does not reference the unstable channel", () => {
    for (const f of [".github/workflows/release.yml", ".github/workflows/release-stable.yml"]) {
      expect(readFileSync(f, "utf8").toLowerCase()).not.toContain("unstable");
    }
  });
  it("stable build injects the tag version", () => {
    expect(stable()).toMatch(/sync-manifest-version\.mjs/);
  });
});
