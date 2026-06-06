import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const cssPath = join(dirname(fileURLToPath(import.meta.url)), "index.css");
const css = readFileSync(cssPath, "utf8");

const sharedAppWindowTokens = [
  "--app-glass-tint",
  "--scroll-fade-tint",
  "--background",
  "--foreground",
  "--primary",
  "--ring",
  "--border",
  "--muted",
  "--muted-foreground",
];

function platformBlock(platform: "windows" | "linux"): string {
  const start = `[data-platform="${platform}"] .app-window {`;
  const end = `[data-platform="${platform === "windows" ? "linux" : "macos"}"] .app-window`;
  const startIndex = css.indexOf(start);
  expect(startIndex).toBeGreaterThan(-1);

  const endIndex = css.indexOf(end, startIndex + start.length);
  expect(endIndex).toBeGreaterThan(startIndex);

  return css.slice(startIndex, endIndex);
}

describe("app-window platform CSS parity", () => {
  it("defines Windows Fluent tokens on .app-window", () => {
    const block = platformBlock("windows");

    for (const token of sharedAppWindowTokens) {
      expect(block).toContain(token);
    }

    expect(block).toContain("#005fb8");
    expect(block).toContain("#f3f3f3");
  });

  it("defines Linux Adwaita tokens on .app-window", () => {
    const block = platformBlock("linux");

    for (const token of sharedAppWindowTokens) {
      expect(block).toContain(token);
    }

    expect(block).toContain("#3584e4");
    expect(block).toContain("#fafafa");
  });

  it("keeps Linux backdrop-filter glass on .app-window", () => {
    expect(css).toMatch(/\.app-window[\s\S]*backdrop-filter:\s*blur\(24px\)/);
    expect(css).toContain('[data-platform="macos"] .app-window');
    expect(css).toMatch(/\[data-platform="macos"\] \.app-window[\s\S]*backdrop-filter:\s*none/);
  });

  it("scopes about-window glass tint per platform", () => {
    expect(css).toContain('[data-platform="windows"] .app-window--about');
    expect(css).toContain('[data-platform="linux"] .app-window--about');
  });

  it("uses opaque document backgrounds for Linux tray and app windows", () => {
    expect(css).toContain('[data-platform="linux"] html[data-tray-panel]');
    expect(css).toContain('[data-platform="linux"] html[data-app-window]');
    expect(css).toMatch(
      /\[data-platform="linux"\] html\[data-tray-panel\][\s\S]*background-color:\s*light-dark\(#fafafa, #242424\)/,
    );
  });

  it("applies app-window document backgrounds to widget windows", () => {
    expect(css).toContain("html[data-widget-window]");
    expect(css).toContain("html[data-widget-window] body");
    expect(css).toContain("html[data-widget-window] #root");
    expect(css).toContain('[data-platform="linux"] html[data-widget-window]');
  });

  it("shows centered overlay titlebar only on macOS app windows", () => {
    expect(css).toMatch(/\.app-window-titlebar \{[\s\S]*display:\s*none/);
    expect(css).toContain('[data-platform="macos"] .app-window-titlebar');
    expect(css).toContain(".app-window-titlebar__drag");
    expect(css).toContain(".app-window-titlebar__title");
    expect(css).toMatch(/\.app-window-titlebar__title[\s\S]*pointer-events:\s*none/);
  });
});
