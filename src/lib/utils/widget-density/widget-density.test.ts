import { describe, expect, it } from "vitest";

import { widgetDensityClasses, type WidgetDensity } from "./widget-density";

describe("widgetDensityClasses", () => {
  const densities: WidgetDensity[] = ["compact", "normal", "expanded"];

  it.each(densities)("returns layout classes for %s density", (density) => {
    const classes = widgetDensityClasses(density);
    expect(classes.root).toContain("flex");
    expect(classes.card).toBeTruthy();
    expect(classes.title).toBeTruthy();
  });

  it("uses smaller typography for compact mode", () => {
    const compact = widgetDensityClasses("compact");
    const expanded = widgetDensityClasses("expanded");
    expect(compact.title).toContain("text-xs");
    expect(expanded.title).toContain("text-base");
  });
});
