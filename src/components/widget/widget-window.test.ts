import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

describe("WidgetWindow", () => {
  it("uses the tray panel usage model instead of the old standalone card layout", () => {
    const source = readFileSync(resolve("src/components/widget/widget-window.tsx"), "utf8");

    expect(source).toContain("useTrayPanelState");
    expect(source).toContain("UsageSnapshotsPanel");
    expect(source).toContain("TrayPanelFooter");
    expect(source).not.toContain("Card");
  });

  it("syncs its native height to the shared tray panel content", () => {
    const source = readFileSync(resolve("src/components/widget/widget-window.tsx"), "utf8");

    expect(source).toContain('useTrayPanelHeight(layoutRef, selectedTab, { target: "widget" })');
    expect(source).not.toContain("h-screen");
  });

  it("does not add an extra opaque outer widget surface", () => {
    const source = readFileSync(resolve("src/components/widget/widget-window.tsx"), "utf8");
    expect(source).not.toContain("bg-background flex h-full");
    expect(source).toContain("TrayPanelShell");
  });

  it("keeps tauri widget defaults aligned with rust widget builder", () => {
    const config = JSON.parse(readFileSync(resolve("src-tauri/tauri.conf.json"), "utf8"));
    const widget = config.app.windows.find(
      (window: { label: string }) => window.label === "widget",
    );
    expect(widget.width).toBe(360);
    expect(widget.height).toBe(420);
    expect(widget.minWidth).toBe(320);
    expect(widget.maxWidth).toBe(480);
  });
});
