import type { ProviderId } from "@/lib/schemas/usage";

const PROVIDER_LABELS: Record<ProviderId, string> = {
  codex: "Codex",
  claude: "Claude",
  cursor: "Cursor",
  gemini: "Gemini",
  copilot: "Copilot",
  opencode: "OpenCode",
  "opencode-go": "OpenCode Go",
  antigravity: "Antigravity",
  factory: "Factory",
  zai: "Z.ai",
  kiro: "Kiro",
  augment: "Augment",
};

export function getProviderLabel(provider: ProviderId): string {
  return PROVIDER_LABELS[provider];
}
