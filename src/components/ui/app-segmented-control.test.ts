import { describe, expect, it } from "vitest";

import { usesPageTabIndicators } from "@/components/ui/app-segmented-control";

describe("usesPageTabIndicators", () => {
  it("enables GSAP hover/active pills for page tabs", () => {
    expect(usesPageTabIndicators("page-tabs")).toBe(true);
  });

  it("disables GSAP hover/active pills for inline controls", () => {
    expect(usesPageTabIndicators("inline")).toBe(false);
  });
});
