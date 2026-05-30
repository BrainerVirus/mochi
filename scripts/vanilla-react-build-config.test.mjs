import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

function readProjectFile(path) {
  return readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
}

describe("vanilla React desktop build config", () => {
  it("does not depend on TanStack Start or Nitro server output", () => {
    const packageJson = JSON.parse(readProjectFile("package.json"));
    const viteConfig = readProjectFile("vite.config.ts");

    expect(packageJson.dependencies).not.toHaveProperty("@tanstack/react-start");
    expect(packageJson.scripts.start).not.toContain("dist/server");
    expect(viteConfig).not.toContain("@tanstack/react-start");
    expect(viteConfig).not.toContain("tanstackStart");
  });
});
