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
    });
  });
}

describe(".github/workflows/release-stable.yml", () => {
  const source = readFileSync(".github/workflows/release-stable.yml", "utf8");

  it("publishes stable feeds to GitHub Pages", () => {
    expect(source).toContain("actions/deploy-pages@v5");
    expect(source).toContain("curl --fail");
  });
});

describe(".github/workflows/release-unstable.yml", () => {
  const source = readFileSync(".github/workflows/release-unstable.yml", "utf8");

  it("does not deploy to GitHub Pages", () => {
    expect(source).not.toContain("actions/deploy-pages@v5");
    expect(source).not.toContain("actions/upload-pages-artifact@v5");
  });
});

describe("linux window experiment cleanup", () => {
  it("does not publish temporary linux window experiment controls", () => {
    const workflow = readFileSync(".github/workflows/release-unstable.yml", "utf8");
    const buildScript = readFileSync("src-tauri/build.rs", "utf8");
    const policy = readFileSync("src-tauri/src/window_policy.rs", "utf8");

    expect(workflow).not.toContain("linux_window_experiment");
    expect(workflow).not.toContain("MOCHI_LINUX_WINDOW_EXPERIMENT");
    expect(workflow).not.toContain("Linux window experiment:");
    expect(buildScript).not.toContain("MOCHI_LINUX_WINDOW_EXPERIMENT");
    expect(policy).not.toContain("MOCHI_LINUX_WINDOW_EXPERIMENT");
    expect(policy).not.toContain("LinuxWindowExperiment");
  });
});
