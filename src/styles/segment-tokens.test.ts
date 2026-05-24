import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const cssPath = join(dirname(fileURLToPath(import.meta.url)), "index.css");
const css = readFileSync(cssPath, "utf8");

describe("segment indicator tokens", () => {
  it("uses grayish white mixes for page-tab hover and active pills", () => {
    expect(css).toMatch(
      /--app-segment-hover:[\s\S]*color-mix\(in oklab, #ffffff 55%, transparent\)/,
    );
    expect(css).toMatch(
      /--app-segment-active:[\s\S]*color-mix\(in oklab, #ffffff 92%, transparent\)/,
    );
    expect(css).toMatch(
      /--tray-segment-hover:[\s\S]*color-mix\(in oklab, #ffffff 55%, transparent\)/,
    );
    expect(css).toMatch(
      /--tray-segment-active:[\s\S]*color-mix\(in oklab, #ffffff 92%, transparent\)/,
    );
  });

  it("keeps primary tints for inline segmented controls", () => {
    expect(css).toMatch(
      /--app-segment-inline-selected:[\s\S]*color-mix\(in oklab, var\(--primary\) 14%, transparent\)/,
    );
  });

  it("sets macOS native primary on app-window and tray-panel shells", () => {
    expect(css).toMatch(
      /\[data-platform="macos"\] \.app-window[\s\S]*--primary: light-dark\(#007aff, #0a84ff\)/,
    );
    expect(css).toMatch(
      /\[data-platform="macos"\] \.tray-panel[\s\S]*--primary: light-dark\(#007aff, #0a84ff\)/,
    );
  });

  it("scopes portaled dialog tokens to native shells", () => {
    expect(css).toContain('html[data-app-window] [data-slot="dialog-content"]');
    expect(css).toContain('html[data-tray-panel] [data-slot="dialog-content"]');
    expect(css).toMatch(
      /html\[data-app-window\] \[data-slot="dialog-content"\][\s\S]*--primary: light-dark\(#007aff, #0a84ff\)/,
    );
  });

  it("keeps tray page-tab segment radii fixed outside .app-window --radius", () => {
    expect(css).toContain("--app-segment-radius-track: 1.5rem");
    expect(css).toContain("--app-segment-radius-indicator: 1.2rem");
    expect(css).toContain("@utility rounded-app-segment-track");
    expect(css).toContain("@utility rounded-app-segment-indicator");

    const appWindowBlock = css.match(/\.app-window \{[\s\S]*?\n\}/)?.[0] ?? "";
    expect(appWindowBlock).not.toContain("--app-segment-radius-track");
    expect(appWindowBlock).not.toContain("--app-segment-radius-indicator");
  });
});
