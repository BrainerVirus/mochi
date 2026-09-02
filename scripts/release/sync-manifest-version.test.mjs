import { describe, expect, it } from "vitest";
import { setPackageVersion, setCargoVersion, setTauriVersion } from "./sync-manifest-version.mjs";

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
});
