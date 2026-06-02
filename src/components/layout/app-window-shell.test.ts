import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { shouldRenderOverlayTitlebar } from "./app-window-titlebar-policy";

describe("AppWindowShell platform chrome", () => {
  it("renders the custom draggable overlay only on macOS", () => {
    expect(shouldRenderOverlayTitlebar("macos")).toBe(true);
    expect(shouldRenderOverlayTitlebar("linux")).toBe(false);
    expect(shouldRenderOverlayTitlebar("windows")).toBe(false);
    expect(shouldRenderOverlayTitlebar("unknown")).toBe(false);
  });

  it("keeps the Tauri drag-region element behind the macOS-only guard", () => {
    const source = readFileSync(resolve("src/components/layout/app-window-shell.tsx"), "utf8");

    expect(source).toContain("shouldRenderOverlayTitlebar(platform)");
    expect(source).toContain("data-tauri-drag-region");
  });
});
