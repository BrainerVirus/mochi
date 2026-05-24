import { describe, expect, it } from "vitest";

import { buildTrayUpdateFooterItems } from "./tray-update-footer-items";

describe("buildTrayUpdateFooterItems", () => {
  it("includes highlighted update row when an update is available", () => {
    const items = buildTrayUpdateFooterItems({
      updateAvailable: true,
      updateVersion: "0.2.0",
      onOpenUpdate: () => {},
      onCheckUpdates: () => {},
    });

    const updateItem = items.find((item) => item.id === "update-available");
    expect(updateItem?.highlight).toBe(true);
    expect(updateItem?.label).toContain("0.2.0");
  });

  it("includes check-for-updates row when no update is pending", () => {
    const items = buildTrayUpdateFooterItems({
      updateAvailable: false,
      onOpenUpdate: () => {},
      onCheckUpdates: () => {},
    });

    expect(items.some((item) => item.id === "check-updates")).toBe(true);
    expect(items.some((item) => item.id === "update-available")).toBe(false);
  });
});
