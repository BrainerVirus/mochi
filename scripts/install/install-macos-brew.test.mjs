import { readFileSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

const brewScript = readFileSync(path.join(import.meta.dirname, "install-macos-brew.sh"), "utf8");

describe("install-macos-brew.sh usage", () => {
  it("does not advertise a release-tag argument (the brew path always installs stable)", () => {
    const usageLine = brewScript.split("\n").find((line) => line.startsWith("# Usage:"));
    expect(usageLine).toBeDefined();
    expect(usageLine).not.toContain("[release-tag]");
  });
});
