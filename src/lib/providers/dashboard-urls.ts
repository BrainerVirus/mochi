import type { ProviderId } from "@/lib/schemas/usage";

export interface ProviderExternalLinks {
  dashboardUrl: string | null;
  statusPageUrl: string | null;
}

/** Dashboard and status URLs aligned with CodexBar provider metadata. */
export const PROVIDER_EXTERNAL_LINKS: Record<ProviderId, ProviderExternalLinks> = {
  codex: {
    dashboardUrl: "https://chatgpt.com/codex/settings/usage",
    statusPageUrl: "https://status.openai.com/",
  },
  claude: {
    dashboardUrl: "https://claude.ai/settings/usage",
    statusPageUrl: "https://status.claude.com/",
  },
  cursor: {
    dashboardUrl: "https://cursor.com/dashboard?tab=usage",
    statusPageUrl: "https://status.cursor.com",
  },
  gemini: {
    dashboardUrl: "https://gemini.google.com",
    statusPageUrl: null,
  },
  copilot: {
    dashboardUrl: "https://github.com/settings/copilot",
    statusPageUrl: "https://www.githubstatus.com/",
  },
  opencode: {
    dashboardUrl: "https://opencode.ai",
    statusPageUrl: null,
  },
  "opencode-go": {
    dashboardUrl: "https://opencode.ai",
    statusPageUrl: null,
  },
  antigravity: {
    dashboardUrl: null,
    statusPageUrl: null,
  },
  factory: {
    dashboardUrl: "https://app.factory.ai/settings/billing",
    statusPageUrl: "https://status.factory.ai",
  },
  zai: {
    dashboardUrl: "https://z.ai/manage-apikey/subscription",
    statusPageUrl: null,
  },
  kiro: {
    dashboardUrl: "https://app.kiro.dev/account/usage",
    statusPageUrl: null,
  },
  augment: {
    dashboardUrl: "https://app.augmentcode.com/account/subscription",
    statusPageUrl: null,
  },
};

export function getProviderExternalLinks(provider: ProviderId): ProviderExternalLinks {
  return PROVIDER_EXTERNAL_LINKS[provider];
}
