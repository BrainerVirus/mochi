import { describe, expect, it } from "vitest";

import { getTrayTabChevronButtonClassName } from "@/components/tray/tray-tab-chevron-class-name";

describe("getTrayTabChevronButtonClassName", () => {
  it("keeps overflow chevrons ghost-style and visually legible", () => {
    const className = getTrayTabChevronButtonClassName(true);

    expect(className).toContain("bg-transparent");
    expect(className).toContain("text-foreground");
    expect(className).toContain("hover:bg-transparent");
    expect(className).not.toContain("bg-background/85");
    expect(className).not.toContain("ring-1");
    expect(className).not.toContain("shadow-sm");
    expect(className).not.toContain("backdrop-blur");
  });

  it("disables pointer events while hidden", () => {
    expect(getTrayTabChevronButtonClassName(false)).toContain("pointer-events-none");
  });
});
