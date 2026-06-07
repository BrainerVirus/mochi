import { describe, expect, it } from "vitest";

import { getProviderExternalLinks, PROVIDER_EXTERNAL_LINKS } from "./dashboard-urls";

describe("PROVIDER_EXTERNAL_LINKS", () => {
  it("includes CodexBar dashboard and status URLs for codex", () => {
    expect(PROVIDER_EXTERNAL_LINKS.codex).toEqual({
      dashboardUrl: "https://chatgpt.com/codex/settings/usage",
      statusPageUrl: "https://status.openai.com/",
    });
  });

  it("omits links for providers without CodexBar metadata", () => {
    expect(getProviderExternalLinks("antigravity")).toEqual({
      dashboardUrl: null,
      statusPageUrl: null,
    });
  });
});
