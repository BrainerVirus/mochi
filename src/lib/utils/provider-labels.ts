import type { ProviderId } from "@/lib/schemas/usage";

const PROVIDER_LABELS: Record<ProviderId, string> = {
  codex: "Codex",
  claude: "Claude",
  cursor: "Cursor",
  gemini: "Gemini",
  copilot: "Copilot",
  antigravity: "Antigravity",
  factory: "Factory",
  zai: "Z.ai",
  kiro: "Kiro",
  augment: "Augment",
};

export function getProviderLabel(provider: ProviderId): string {
  return PROVIDER_LABELS[provider];
}
