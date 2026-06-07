import { describe, expect, it } from "vitest";

import { trayPanelDividerClassName, trayPanelSpacing } from "./tray-panel-spacing";

describe("trayPanelSpacing", () => {
  it("uses consistent divider rhythm for inter-group gaps", () => {
    expect(trayPanelSpacing.dividerBefore).toBe("pt-2");
    expect(trayPanelSpacing.dividerAfter).toBe("mb-1");
    expect(trayPanelSpacing.meterGap).toBe("gap-3");
    expect(trayPanelSpacing.headerToMeters).toBe("mt-2");
  });

  it("adds horizontal inset only for panel-edge dividers", () => {
    expect(trayPanelDividerClassName()).toBe("pt-2 pb-0");
    expect(trayPanelDividerClassName(true)).toBe("px-3 pt-2 pb-0");
  });
});
