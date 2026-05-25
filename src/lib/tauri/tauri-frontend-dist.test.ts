import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

const PUBLIC_DIST = join(process.cwd(), ".output/public");
const SHELL_PATH = join(PUBLIC_DIST, "index.html");

describe("Tauri frontend dist packaging", () => {
  it("includes the SPA shell expected by WebviewUrl::App(index.html)", () => {
    expect(
      existsSync(SHELL_PATH),
      "Missing .output/public/index.html — run `pnpm build` before tests.",
    ).toBe(true);

    const shell = readFileSync(SHELL_PATH, "utf8");
    expect(shell.length).toBeGreaterThan(0);
    expect(shell).toMatch(/<html[\s>]/i);
  });
});
