import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { collectUpdaterArtifacts } from "./collect-updater-artifacts.mjs";

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

  it("derives an unstable version that is newer than the recovery versions", async () => {
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

    const manifest = await collectUpdaterArtifacts({
      artifactRoot: root,
      channel: "unstable",
      tagName: "unstable-20260606.123456",
      unstableBaseVersion: "0.2.1",
      releaseBaseUrl:
        "https://github.com/BrainerVirus/mochi/releases/download/unstable-20260606.123456",
      releaseNotesPath: join(root, "missing-notes.md"),
      outputPath: join(root, "updater-feed.json"),
      pubDate: "2026-06-06T12:34:56.000Z",
    });

    expect(manifest.latestVersion).toBe("0.2.1-unstable.20260606.123456");
    expect(manifest.versions).toContain("0.1.7");
    expect(manifest.versions).toContain("0.2.0");
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
        channel: "unstable",
        tagName: "unstable-20260606.123456",
        unstableBaseVersion: "0.2.1",
        releaseBaseUrl:
          "https://github.com/BrainerVirus/mochi/releases/download/unstable-20260606.123456",
        releaseNotesPath: join(root, "missing-notes.md"),
        outputPath: join(root, "updater-feed.json"),
        pubDate: "2026-06-06T12:34:56.000Z",
      }),
    ).rejects.toThrow("missing updater artifact for darwin-aarch64");
  });
});
