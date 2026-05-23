import { z } from "zod";

import { ProviderIdSchema } from "./usage";

export const UpdateChannelSchema = z.enum(["stable", "unstable"]);

export type UpdateChannel = z.infer<typeof UpdateChannelSchema>;

export const ProviderConfigSchema = z.object({
  cookie_source: z.string().optional(),
  manual_cookie: z.string().optional(),
  api_key: z.string().optional(),
  admin_api_key: z.string().optional(),
  history_window_days: z.number().int().min(1).max(365).optional(),
  region_host: z.string().optional(),
  token_account: z.string().optional(),
});

export type ProviderConfig = z.infer<typeof ProviderConfigSchema>;

export const MochiSettingsSchema = z.object({
  update_channel: UpdateChannelSchema,
  refresh_interval_seconds: z.number().int().min(30).max(86_400),
  enabled_providers: z.array(ProviderIdSchema),
  show_notifications: z.boolean(),
  provider_configs: z.record(z.string(), ProviderConfigSchema).default({}),
});

export type MochiSettings = z.infer<typeof MochiSettingsSchema>;

export const PROVIDER_LABELS: Record<z.infer<typeof ProviderIdSchema>, string> = {
  codex: "Codex",
  claude: "Claude",
  cursor: "Cursor",
  gemini: "Gemini",
  copilot: "Copilot",
  opencode: "OpenCode",
  "opencode-go": "OpenCode Go",
  antigravity: "Antigravity",
  factory: "Factory/Droid",
  zai: "z.ai",
  kiro: "Kiro",
  augment: "Augment",
};

export const ALL_PROVIDER_IDS = ProviderIdSchema.options;

/** Matches `MochiSettings::default()` in `src-tauri/src/settings/mod.rs`. */
export const DEFAULT_MOCHI_SETTINGS = MochiSettingsSchema.parse({
  update_channel: "stable",
  refresh_interval_seconds: 300,
  enabled_providers: ALL_PROVIDER_IDS,
  show_notifications: true,
  provider_configs: {},
});
