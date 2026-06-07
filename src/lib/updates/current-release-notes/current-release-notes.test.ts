import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/tauri/commands", () => ({
  appVersion: vi.fn<() => Promise<string>>(() => Promise.resolve("0.2.0")),
}));

vi.mock("@/lib/updates/release-notes-cache", () => ({
  cacheReleaseNotes: vi.fn<(entry: unknown) => void>(),
}));

import { appVersion } from "@/lib/tauri/commands";
import { cacheReleaseNotes } from "@/lib/updates/release-notes-cache";

import {
  fetchCurrentReleaseNotes,
  githubReleaseTagForChannel,
  releaseNotesCacheKeyForChannel,
} from "./current-release-notes";

describe("current release notes", () => {
  beforeEach(() => {
    vi.mocked(appVersion).mockResolvedValue("0.2.0");
    vi.mocked(cacheReleaseNotes).mockClear();
    vi.stubGlobal(
      "fetch",
      vi.fn<
        () => Promise<{
          ok: boolean;
          json: () => Promise<{ tag_name: string; body: string }>;
        }>
      >(() =>
        Promise.resolve({
          ok: true,
          json: () =>
            Promise.resolve({
              tag_name: "v0.2.0",
              body: "### What's changed\n- Fix tray\n\n### Install stable\n- macOS: `curl example`",
            }),
        }),
      ),
    );
  });

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

  it("caches sanitized installed-version release notes for stable", async () => {
    await expect(fetchCurrentReleaseNotes("stable")).resolves.toMatchObject({
      version: "v0.2.0",
      notes: "### What's changed\n- Fix tray",
      source: "installed-release",
    });

    expect(fetch).toHaveBeenCalledWith(
      "https://api.github.com/repos/BrainerVirus/mochi/releases/tags/v0.2.0",
    );
    expect(cacheReleaseNotes).toHaveBeenCalledWith(
      expect.objectContaining({
        version: "v0.2.0",
        notes: "### What's changed\n- Fix tray",
        channel: "stable",
        source: "installed-release",
      }),
    );
  });
});
