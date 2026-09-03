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
  it("sets the mochi package version in Cargo.lock keeping other packages", () => {
    const src =
      '[[package]]\nname = "other"\nversion = "1.0.0"\n\n[[package]]\nname = "mochi"\nversion = "0.0.1"\n';
    const out = setCargoLockVersion(src, "9.9.9");
    expect(out).toContain('name = "mochi"\nversion = "9.9.9"');
    expect(out).toContain('name = "other"\nversion = "1.0.0"');
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
