import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

describe("native desktop window controls", () => {
  it("keeps settings as a normal decorated taskbar window", () => {
    const source = readFileSync(resolve("src-tauri/src/tray/panel.rs"), "utf8");
    const appWindowSection = source.slice(
      source.indexOf("pub fn prepare_app_window"),
      source.indexOf("fn ensure_main_panel_window"),
    );

    expect(appWindowSection).not.toContain("prevent_close");
    expect(appWindowSection).not.toContain("set_skip_taskbar(true)");
    expect(appWindowSection).not.toContain(".skip_taskbar(true)");
  });

  it("lets the widget native close button close the native window", () => {
    const source = readFileSync(resolve("src-tauri/src/widget/commands.rs"), "utf8");

    expect(source).not.toContain("WindowEvent::CloseRequested");
    expect(source).not.toContain("prevent_close");
    expect(source).not.toContain("close_requested -> hide");
  });

  it("does not reset native decorations after decorated windows are created", () => {
    const source = readFileSync(resolve("src-tauri/src/linux_window_controls.rs"), "utf8");

    expect(source).not.toContain("set_decorations(true)");
    expect(source).not.toContain("set_resizable(true)");
    expect(source).toContain("linux_builder_decorations");
    expect(source).toContain("linux_builder_resizable");
  });

  it("does not precreate the decorated widget from tauri config", () => {
    const config = JSON.parse(readFileSync(resolve("src-tauri/tauri.conf.json"), "utf8"));
    const widget = config.app.windows.find(
      (window: { label: string }) => window.label === "widget",
    );

    expect(widget).toBeUndefined();

    const source = readFileSync(resolve("src-tauri/src/widget/commands.rs"), "utf8");
    expect(source).toContain(".decorations(true)");
    expect(source).toContain(".resizable(true)");
  });

  it("does not precreate decorated linux app windows for on-demand-visible", () => {
    const lib = readFileSync(resolve("src-tauri/src/lib.rs"), "utf8");
    const policy = readFileSync(resolve("src-tauri/src/window_policy.rs"), "utf8");

    expect(policy).toContain("OnDemandVisible");
    expect(lib).toContain("should_precreate_decorated_windows_at_startup");
  });

  it("widget config is not the permanent linux on-demand creation source", () => {
    const commands = readFileSync(resolve("src-tauri/src/widget/commands.rs"), "utf8");

    expect(commands).toContain("build_widget_window");
    expect(commands).toContain("DecoratedWindowInitialVisibility::Visible");
    expect(commands).toContain('record_widget_window_lifecycle(&window, "created", "on-demand"');
  });
});
