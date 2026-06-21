import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

for (const workflow of [
  ".github/workflows/release-unstable.yml",
  ".github/workflows/publish-updater-pages.yml",
  ".github/workflows/republish-updater-pages.yml",
]) {
  describe(workflow, () => {
    const source = readFileSync(workflow, "utf8");

    it("builds and validates updater feeds", () => {
      expect(source).toContain("scripts/release/collect-updater-artifacts.mjs");
      expect(source).toContain("scripts/release/build-updater-feed.mjs");
      expect(source).toContain("scripts/release/validate-updater-feed.mjs");
    });
  });
}

describe(".github/workflows/release-stable.yml", () => {
  const source = readFileSync(".github/workflows/release-stable.yml", "utf8");

  it("requires updater signing secrets before building", () => {
    expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY");
    expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
    expect(source).toContain("MOCHI_UPDATER_PUBLIC_KEY");
  });

  it("uses ad-hoc macOS signing without Apple Developer secrets", () => {
    expect(source).toContain("APPLE_SIGNING_IDENTITY");
    expect(source).toContain("ad-hoc signing");
    expect(source).not.toContain("APPLE_CERTIFICATE");
    expect(source).not.toContain("verify-macos-signing-env.sh");
  });

  it("delegates Pages publish to reusable workflow on main", () => {
    expect(source).toContain("publish-updater-pages.yml@main");
    expect(source).not.toContain("actions/deploy-pages@v5");
    expect(source).not.toContain("environment:");
  });
});

describe(".github/workflows/release-unstable.yml", () => {
  const source = readFileSync(".github/workflows/release-unstable.yml", "utf8");

  it("requires updater signing secrets before building", () => {
    expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY");
    expect(source).toContain("TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
    expect(source).toContain("MOCHI_UPDATER_PUBLIC_KEY");
  });

  it("does not deploy to GitHub Pages", () => {
    expect(source).not.toContain("actions/deploy-pages@v5");
    expect(source).not.toContain("actions/upload-pages-artifact@v5");
    expect(source).not.toContain("pages: write");
    expect(source).not.toContain("release-pages");
  });
});

describe(".github/workflows/publish-updater-pages.yml", () => {
  const source = readFileSync(".github/workflows/publish-updater-pages.yml", "utf8");

  it("retries live feed validation after deploy", () => {
    expect(source).toContain("validate-published-feeds.mjs");
    expect(source).toContain("release-pages");
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
