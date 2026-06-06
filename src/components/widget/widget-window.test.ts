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

    expect(source).not.toContain("useTrayPanelHeight");
    expect(source).not.toContain("h-screen");
  });

  it("renders a native-window shell instead of nested tray panel chrome", () => {
    const source = readFileSync(resolve("src/components/widget/widget-window.tsx"), "utf8");
    expect(source).not.toContain("bg-background flex h-full");
    expect(source).not.toContain("TrayPanelShell");
    expect(source).not.toContain("trayPanelShellClassName");
    expect(source).toContain("overflow-y-auto");
    expect(source).toContain("data-widget-window-shell");
    expect(source).toContain("max-w-[360px]");
  });

  it("keeps tauri widget defaults aligned with rust widget builder", () => {
    const config = JSON.parse(readFileSync(resolve("src-tauri/tauri.conf.json"), "utf8"));
    const widget = config.app.windows.find(
      (window: { label: string }) => window.label === "widget",
    );
    expect(widget.width).toBe(360);
    expect(widget.height).toBe(420);
    expect(widget.minWidth).toBe(320);
    expect(widget).not.toHaveProperty("maxWidth");
    expect(widget.alwaysOnTop).toBe(false);
  });

  it("does not force the decorated widget to stay always on top", () => {
    const source = readFileSync(resolve("src-tauri/src/widget/commands.rs"), "utf8");

    expect(source).not.toContain(".always_on_top(true)");
    expect(source).not.toContain("set_always_on_top(true)");
    expect(source).not.toContain(".max_inner_size(");
    expect(source).not.toContain("set_max_size(");
  });
});
