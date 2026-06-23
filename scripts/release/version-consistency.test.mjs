import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("release versions", () => {
  it("keeps package, Cargo, and Tauri versions in sync", () => {
    const packageVersion = JSON.parse(readFileSync("package.json", "utf8")).version;
    const tauriVersion = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")).version;
    const cargoVersion = readFileSync("src-tauri/Cargo.toml", "utf8").match(
      /^version = "([^"]+)"/m,
    )?.[1];

    expect([packageVersion, cargoVersion, tauriVersion]).toEqual([
      tauriVersion,
      tauriVersion,
      tauriVersion,
    ]);
  });
});
