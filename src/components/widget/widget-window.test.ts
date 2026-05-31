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
});
