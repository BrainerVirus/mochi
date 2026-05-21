import { describe, expect, it } from "vitest";

import { ProviderIdSchema } from "@/lib/schemas/usage";

import { PROVIDER_BRAND_SVGS } from "./provider-icon-sources";

describe("PROVIDER_BRAND_SVGS", () => {
  it("includes a bundled SVG for every built-in provider", () => {
    for (const provider of ProviderIdSchema.options) {
      expect(PROVIDER_BRAND_SVGS[provider].length).toBeGreaterThan(0);
      expect(PROVIDER_BRAND_SVGS[provider]).toContain("<svg");
    }
  });
});
