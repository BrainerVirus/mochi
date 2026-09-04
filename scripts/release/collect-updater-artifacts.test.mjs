import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { collectUpdaterArtifacts, resolveFeedVersions } from "./collect-updater-artifacts.mjs";

async function writeArtifact(root, relativePath, signature) {
  const path = join(root, relativePath);
  await mkdir(join(path, ".."), { recursive: true });
  await writeFile(path, "artifact");
  await writeFile(`${path}.sig`, signature);
}

describe("collectUpdaterArtifacts", () => {
  it("derives version, pubDate, release URLs, signatures, and stable channel", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    await writeArtifact(
      root,
      "updater-bundle-macos-26-macos-arm64/aarch64-apple-darwin/release/bundle/macos/Mochi.app.tar.gz",
      "sig-darwin-arm",
    );
    await writeArtifact(
      root,
      "updater-bundle-macos-26-intel-macos-x64/x86_64-apple-darwin/release/bundle/macos/Mochi.app.tar.gz",
      "sig-darwin-x64",
    );
    await writeArtifact(
      root,
      "updater-bundle-ubuntu-24.04-linux-x64/release/bundle/appimage/Mochi_0.2.1_amd64.AppImage",
      "sig-linux",
    );
    await writeArtifact(
      root,
      "updater-bundle-windows-2025-vs2026-windows-x64/release/bundle/nsis/Mochi_0.2.1_x64-setup.exe",
      "sig-windows",
    );

    const manifestPath = join(root, "updater-feed.json");
    const manifest = await collectUpdaterArtifacts({
      artifactRoot: root,
      channel: "stable",
      tagName: "v0.2.1",
      releaseBaseUrl: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1",
      releaseNotesPath: join(root, "missing-notes.md"),
      outputPath: manifestPath,
      pubDate: "2026-06-06T12:34:56.000Z",
    });

    expect(manifest.latestVersion).toBe("0.2.1");
    expect(manifest.channels).toEqual(["stable"]);
    expect(manifest.pubDate).toBe("2026-06-06T12:34:56.000Z");
    expect(manifest.artifacts["darwin-aarch64"].signature).toBe("sig-darwin-arm");
    expect(manifest.artifacts["darwin-aarch64"].url).toBe(
      "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi.app.tar.gz",
    );
    expect(manifest.artifacts["linux-x86_64"].url).toBe(
      "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_0.2.1_amd64.AppImage",
    );
    expect(manifest.artifacts["windows-x86_64"].url).toBe(
      "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1/Mochi_0.2.1_x64-setup.exe",
    );
    expect(JSON.parse(await readFile(manifestPath, "utf8"))).toEqual(manifest);
  });

  it("rejects removed or unknown channels", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    const legacyChannel = "unstable";
    await expect(
      collectUpdaterArtifacts({
        artifactRoot: root,
        channel: legacyChannel,
        tagName: "0.2.1-rc.1",
        releaseBaseUrl: "https://github.com/BrainerVirus/mochi/releases/download/0.2.1-rc.1",
        releaseNotesPath: join(root, "missing-notes.md"),
        outputPath: join(root, "updater-feed.json"),
        pubDate: "2026-06-06T12:34:56.000Z",
      }),
    ).rejects.toThrow(`unsupported updater channel: ${legacyChannel}`);
  });

  it("accepts flat GitHub Release asset names for republish", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    await writeArtifact(root, "Mochi_aarch64.app.tar.gz", "sig-darwin-arm");
    await writeArtifact(root, "Mochi_x64.app.tar.gz", "sig-darwin-x64");
    await writeArtifact(root, "Mochi_0.2.4_amd64.AppImage", "sig-linux");
    await writeArtifact(root, "Mochi_0.2.4_x64-setup.exe", "sig-windows");

    const manifest = await collectUpdaterArtifacts({
      artifactRoot: root,
      channel: "stable",
      tagName: "v0.2.4",
      releaseBaseUrl: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.4",
      releaseNotesPath: join(root, "missing-notes.md"),
      outputPath: join(root, "updater-feed.json"),
      pubDate: "2026-06-21T12:00:00.000Z",
    });

    expect(manifest.artifacts["darwin-aarch64"].url).toBe(
      "https://github.com/BrainerVirus/mochi/releases/download/v0.2.4/Mochi_aarch64.app.tar.gz",
    );
    expect(manifest.artifacts["windows-x86_64"].url).toBe(
      "https://github.com/BrainerVirus/mochi/releases/download/v0.2.4/Mochi_0.2.4_x64-setup.exe",
    );
  });

  it("fails when an updater signature is missing", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    await mkdir(join(root, "updater-bundle-ubuntu-24.04-linux-x64/release/bundle/appimage"), {
      recursive: true,
    });
    await writeFile(
      join(
        root,
        "updater-bundle-ubuntu-24.04-linux-x64/release/bundle/appimage/Mochi_0.2.1_amd64.AppImage",
      ),
      "artifact",
    );

    await expect(
      collectUpdaterArtifacts({
        artifactRoot: root,
        channel: "stable",
        tagName: "v0.2.1",
        releaseBaseUrl: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.1",
        releaseNotesPath: join(root, "missing-notes.md"),
        outputPath: join(root, "updater-feed.json"),
        pubDate: "2026-06-06T12:34:56.000Z",
      }),
    ).rejects.toThrow("missing updater artifact for darwin-aarch64");
  });

  it("retains feeds for every tagged stable version", async () => {
    expect(
      resolveFeedVersions({
        latestVersion: "0.4.1",
        tags: ["v0.3.0", "v0.4.0", "v0.4.1", "unstable-20260828.1"],
      }),
    ).toEqual(["0.1.7", "0.2.0", "0.3.0", "0.4.0", "0.4.1"]);
  });

  it("falls back to the hardcoded list when git yields no tags", async () => {
    expect(resolveFeedVersions({ latestVersion: "0.2.1", tags: [] })).toEqual([
      "0.1.7",
      "0.2.0",
      "0.2.1",
    ]);
  });

  it("excludes unstable tags and keeps untagged latest versions", async () => {
    expect(
      resolveFeedVersions({ latestVersion: "9.9.9", tags: ["unstable-20260828.1", "v0.3.0"] }),
    ).toEqual(["0.1.7", "0.2.0", "0.3.0", "9.9.9"]);
  });

  it("sorts versions numerically, not lexicographically", async () => {
    expect(resolveFeedVersions({ latestVersion: "0.10.0", tags: ["v0.9.0", "v0.10.0"] })).toEqual([
      "0.1.7",
      "0.2.0",
      "0.9.0",
      "0.10.0",
    ]);
  });

  it("writes retained versions into the manifest", async () => {
    const root = await mkdtemp(join(tmpdir(), "mochi-updater-artifacts-"));
    await writeArtifact(root, "Mochi_aarch64.app.tar.gz", "sig-darwin-arm");
    await writeArtifact(root, "Mochi_x64.app.tar.gz", "sig-darwin-x64");
    await writeArtifact(root, "Mochi_0.4.1_amd64.AppImage", "sig-linux");
    await writeArtifact(root, "Mochi_0.4.1_x64-setup.exe", "sig-windows");

    const manifest = await collectUpdaterArtifacts({
      artifactRoot: root,
      channel: "stable",
      tagName: "v0.4.1",
      releaseBaseUrl: "https://github.com/BrainerVirus/mochi/releases/download/v0.4.1",
      releaseNotesPath: join(root, "missing-notes.md"),
      outputPath: join(root, "updater-feed.json"),
      pubDate: "2026-06-06T12:34:56.000Z",
      tags: ["v0.3.0", "v0.4.0", "v0.4.1"],
    });

    expect(manifest.versions).toEqual(["0.1.7", "0.2.0", "0.3.0", "0.4.0", "0.4.1"]);
  });
});
