import { z } from "zod";

import { ProviderIdSchema } from "./usage";

export const UpdateChannelSchema = z.enum(["stable", "unstable"]);

export type UpdateChannel = z.infer<typeof UpdateChannelSchema>;

export const MochiSettingsSchema = z.object({
  update_channel: UpdateChannelSchema,
  refresh_interval_seconds: z.number().int().min(30).max(86_400),
  enabled_providers: z.array(ProviderIdSchema),
  show_notifications: z.boolean(),
});

export type MochiSettings = z.infer<typeof MochiSettingsSchema>;

export const PROVIDER_LABELS: Record<z.infer<typeof ProviderIdSchema>, string> = {
  codex: "Codex",
  claude: "Claude",
  cursor: "Cursor",
  gemini: "Gemini",
  copilot: "Copilot",
  antigravity: "Antigravity",
  factory: "Factory/Droid",
  zai: "z.ai",
  kiro: "Kiro",
  augment: "Augment",
};

export const ALL_PROVIDER_IDS = ProviderIdSchema.options;
