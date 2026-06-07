import { z } from "zod";

import { normalizeProviderId, ProviderIdSchema } from "@/lib/schemas/usage";

export const UpdateChannelSchema = z.enum(["stable", "unstable"]);

export type UpdateChannel = z.infer<typeof UpdateChannelSchema>;

export const TokenAccountSchema = z.object({
  id: z.string(),
  label: z.string(),
  token: z.string(),
});

export const TokenAccountDataSchema = z.object({
  version: z.number().int(),
  accounts: z.array(TokenAccountSchema),
  activeIndex: z.number().int().nonnegative(),
});

export type TokenAccountData = z.infer<typeof TokenAccountDataSchema>;

export const ProviderConfigSchema = z.object({
  cookie_source: z.string().optional(),
  manual_cookie: z.string().optional(),
  api_key: z.string().optional(),
  admin_api_key: z.string().optional(),
  history_window_days: z.number().int().min(1).max(365).optional(),
  region_host: z.string().optional(),
  token_account: z.string().optional(),
  workspace_id: z.string().optional(),
  token_accounts: TokenAccountDataSchema.optional(),
});

export type ProviderConfig = z.infer<typeof ProviderConfigSchema>;

export const MochiSettingsSchema = z.object({
  update_channel: UpdateChannelSchema,
  refresh_interval_seconds: z.number().int().min(30).max(86_400),
  enabled_providers: z
    .array(z.string())
    .transform((providers) =>
      providers.flatMap((provider) => {
        const normalized = normalizeProviderId(provider);
        return normalized ? [normalized] : [];
      }),
    )
    .pipe(z.array(ProviderIdSchema)),
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
  enabled_providers: [],
  show_notifications: true,
  provider_configs: {},
});
