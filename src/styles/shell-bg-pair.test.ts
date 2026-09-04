import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

// The shell background hex pair lives in three places: the Rust
// SHELL_BG_LIGHT/SHELL_BG_DARK consts (window_background.rs, covered by its
// Rust tests), the index.html critical first-paint CSS, and
// src/styles/index.css. This pins the two web copies to the same pair so a
// one-sided edit (white flash / wrong-color frame) fails loudly.
const stylesDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(stylesDir, "..", "..");
const html = readFileSync(join(repoRoot, "index.html"), "utf8");
const css = readFileSync(join(stylesDir, "index.css"), "utf8");

const LIGHT = "#fafafa";
const DARK = "#242424";

describe("shell background hex pair", () => {
  it("uses the same light/dark pair in index.html critical CSS", () => {
    expect(html).toContain(`background-color: ${LIGHT};`);
    expect(html).toContain(`background-color: ${DARK};`);
    expect(html).toMatch(/prefers-color-scheme:\s*dark[\s\S]*background-color:\s*#242424/);
  });

  it("uses the same light/dark pair in index.css", () => {
    expect(css).toContain(`light-dark(${LIGHT}, ${DARK})`);
  });
});
