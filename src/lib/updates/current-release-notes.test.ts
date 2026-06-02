import { describe, expect, it } from "vitest";

import {
  githubReleaseTagForChannel,
  releaseNotesCacheKeyForChannel,
} from "./current-release-notes";

describe("current release notes", () => {
  it("uses the rolling unstable tag for the unstable channel", () => {
    expect(githubReleaseTagForChannel("unstable", "0.1.5")).toBe("unstable");
  });

  it("uses the current app version tag for stable release notes", () => {
    expect(githubReleaseTagForChannel("stable", "0.1.5")).toBe("v0.1.5");
    expect(githubReleaseTagForChannel("stable", "v0.1.5")).toBe("v0.1.5");
  });

  it("keeps stable and unstable caches separate", () => {
    expect(releaseNotesCacheKeyForChannel("stable")).not.toBe(
      releaseNotesCacheKeyForChannel("unstable"),
    );
  });
});
