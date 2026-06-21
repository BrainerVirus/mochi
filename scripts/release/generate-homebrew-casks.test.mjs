import { describe, expect, it } from "vitest";
import { caskVersionFromTag, pickMacDmgAssets, renderCask } from "./generate-homebrew-casks.mjs";

const stableRelease = {
  tag_name: "v0.2.4",
  assets: [
    {
      name: "Mochi_0.2.4_aarch64.dmg",
      browser_download_url:
        "https://github.com/BrainerVirus/mochi/releases/download/v0.2.4/Mochi_0.2.4_aarch64.dmg",
    },
    {
      name: "Mochi_0.2.4_x64.dmg",
      browser_download_url:
        "https://github.com/BrainerVirus/mochi/releases/download/v0.2.4/Mochi_0.2.4_x64.dmg",
    },
  ],
};

describe("caskVersionFromTag", () => {
  it("strips the v prefix from stable tags", () => {
    expect(caskVersionFromTag("v0.2.4")).toBe("0.2.4");
  });

  it("keeps unstable timestamp tags intact", () => {
    expect(caskVersionFromTag("unstable-20260621.225155")).toBe("unstable-20260621.225155");
  });
});

describe("pickMacDmgAssets", () => {
  it("selects arm64 and x64 dmg assets from a release", () => {
    expect(pickMacDmgAssets(stableRelease)).toEqual({
      arm64: {
        name: "Mochi_0.2.4_aarch64.dmg",
        url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.4/Mochi_0.2.4_aarch64.dmg",
      },
      x64: {
        name: "Mochi_0.2.4_x64.dmg",
        url: "https://github.com/BrainerVirus/mochi/releases/download/v0.2.4/Mochi_0.2.4_x64.dmg",
      },
    });
  });
});

describe("renderCask", () => {
  it("renders a multi-arch stable cask", () => {
    const body = renderCask({
      id: "mochi-desktop",
      name: "Mochi",
      desc: "Cross-platform desktop companion for AI coding tool usage",
      version: "0.2.4",
      homepage: "https://github.com/BrainerVirus/mochi",
      arm64: {
        sha256: "arm-sha",
        url: "https://example.com/Mochi_0.2.4_aarch64.dmg",
      },
      x64: {
        sha256: "x64-sha",
        url: "https://example.com/Mochi_0.2.4_x64.dmg",
      },
    });

    expect(body).toContain('cask "mochi-desktop" do');
    expect(body).toContain('version "0.2.4"');
    expect(body).toContain('sha256 "arm-sha"');
    expect(body).toContain('sha256 "x64-sha"');
    expect(body).toContain("on_arm do");
    expect(body).toContain("on_intel do");
  });
});
