import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vitest";
import {
  setPackageVersion,
  setCargoVersion,
  setCargoLockVersion,
  setTauriVersion,
} from "./sync-manifest-version.mjs";

describe("sync-manifest-version", () => {
  it("sets package.json version", () => {
    const src = '{\n  "name": "mochi",\n  "version": "0.0.1"\n}';
    expect(JSON.parse(setPackageVersion(src, "9.9.9")).version).toBe("9.9.9");
  });
  it("sets Cargo.toml version keeping formatting", () => {
    const src = '[package]\nname = "mochi"\nversion = "0.0.1"\nedition = "2021"\n';
    expect(setCargoVersion(src, "9.9.9")).toContain('version = "9.9.9"');
    expect(setCargoVersion(src, "9.9.9")).toContain('edition = "2021"');
  });
  it("sets tauri.conf.json version", () => {
    const src = '{\n  "version": "0.0.1",\n  "identifier": "app.mochi"\n}';
    expect(JSON.parse(setTauriVersion(src, "9.9.9")).version).toBe("9.9.9");
  });
  it("keeps compact single-line arrays byte-identical in tauri.conf.json", () => {
    const src = [
      "{",
      '  "$schema": "https://schema.tauri.app/config/2",',
      '  "version": "0.2.6",',
      '  "identifier": "app.mochi.Mochi",',
      '  "app": {',
      '    "security": {',
      '      "capabilities": ["default"]',
      "    }",
      "  },",
      '  "bundle": {',
      '    "targets": ["app", "dmg", "msi", "nsis", "appimage", "deb", "rpm"]',
      "  }",
      "}",
      "",
    ].join("\n");
    const out = setTauriVersion(src, "0.4.0");
    expect(out).toContain('"capabilities": ["default"]');
    expect(out).toContain('"targets": ["app", "dmg", "msi", "nsis", "appimage", "deb", "rpm"]');
    expect(out).toContain('"version": "0.4.0"');
    expect(out).not.toContain('"version": "0.2.6"');
    const srcLines = src.split("\n");
    const outLines = out.split("\n");
    expect(outLines.length).toBe(srcLines.length);
    for (const line of srcLines) {
      if (line.includes('"capabilities"') || line.includes('"targets"')) {
        expect(outLines).toContain(line);
      }
    }
  });
  it("keeps every non-version byte identical in package.json", () => {
    const src = '{\n  "name": "mochi",\n  "version": "0.2.6",\n  "private": true\n}\n';
    const out = setPackageVersion(src, "0.4.0");
    expect(out).toContain('"version": "0.4.0"');
    expect(out).not.toContain('"version": "0.2.6"');
    expect(out.split("\n").length).toBe(src.split("\n").length);
    for (const line of src.split("\n")) {
      if (!line.includes('"version"')) {
        expect(out.split("\n")).toContain(line);
      }
    }
  });
  it("sets the mochi package version in Cargo.lock keeping other packages", () => {
    const src =
      '[[package]]\nname = "other"\nversion = "1.0.0"\n\n[[package]]\nname = "mochi"\nversion = "0.0.1"\n';
    const out = setCargoLockVersion(src, "9.9.9");
    expect(out).toContain('name = "mochi"\nversion = "9.9.9"');
    expect(out).toContain('name = "other"\nversion = "1.0.0"');
  });
  it("throws when Cargo.lock has no mochi package entry", () => {
    const src = '[[package]]\nname = "other"\nversion = "1.0.0"\n';
    expect(() => setCargoLockVersion(src, "9.9.9")).toThrow(/mochi/);
  });
  it("rejects non-semver --set values with exit code 2", () => {
    const result = spawnSync(
      process.execPath,
      ["scripts/release/sync-manifest-version.mjs", "--set", "not-semver"],
      {
        encoding: "utf8",
      },
    );
    expect(result.status).toBe(2);
    expect(result.stderr).toContain("usage");
  });
});
