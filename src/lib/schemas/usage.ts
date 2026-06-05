import { z } from "zod";

export const ProviderIdSchema = z.enum([
  "codex",
  "claude",
  "cursor",
  "gemini",
  "copilot",
  "opencode",
  "opencode-go",
  "antigravity",
  "factory",
  "zai",
  "kiro",
  "augment",
]);

export type ProviderId = z.infer<typeof ProviderIdSchema>;

export const ProviderCostSnapshotSchema = z.object({
  used: z.number(),
  limit: z.number(),
  currency_code: z.string(),
  period: z.string().nullable().optional(),
  resets_at: z.string().nullable().optional(),
});

export type ProviderCostSnapshot = z.infer<typeof ProviderCostSnapshotSchema>;

export const UsageWindowSchema = z.object({
  label: z.string(),
  used_percent: z.number(),
  remaining_percent: z.number(),
  resets_at: z.string().nullable(),
});

export type UsageWindow = z.infer<typeof UsageWindowSchema>;

export const ProviderHealthSchema = z.enum(["ok", "stale", "error"]);

export type ProviderHealth = z.infer<typeof ProviderHealthSchema>;

export const FetchAttemptSchema = z.object({
  strategy_id: z.string(),
  succeeded: z.boolean(),
  error: z.string().nullable().optional(),
  attempted_at: z.string(),
});

export type FetchAttempt = z.infer<typeof FetchAttemptSchema>;

export const StatusIndicatorSchema = z.enum([
  "none",
  "minor",
  "major",
  "critical",
  "maintenance",
  "unknown",
]);

export type StatusIndicator = z.infer<typeof StatusIndicatorSchema>;

export const ProviderStatusSchema = z.object({
  indicator: StatusIndicatorSchema,
  description: z.string().nullable().optional(),
  updated_at: z.string().nullable().optional(),
  url: z.string().nullable().optional(),
});

export type ProviderStatus = z.infer<typeof ProviderStatusSchema>;

export const SessionCostSummarySchema = z.object({
  window_days: z.number(),
  input_tokens: z.number(),
  cached_input_tokens: z.number(),
  output_tokens: z.number(),
  session_files_scanned: z.number(),
});

export type SessionCostSummary = z.infer<typeof SessionCostSummarySchema>;

export const UsageSnapshotSchema = z.object({
  provider: ProviderIdSchema,
  primary: UsageWindowSchema,
  secondary: UsageWindowSchema.nullable(),
  tertiary: UsageWindowSchema.nullable().optional(),
  extra_windows: z.array(UsageWindowSchema).default([]),
  provider_cost: ProviderCostSnapshotSchema.nullable().optional(),
  updated_at: z.string(),
  source: z.string(),
  health: ProviderHealthSchema.default("ok"),
  is_stale: z.boolean().default(false),
  error: z.string().nullable().optional(),
  last_fetch_attempt: FetchAttemptSchema.nullable().optional(),
  provider_status: ProviderStatusSchema.nullable().optional(),
  session_cost: SessionCostSummarySchema.nullable().optional(),
});

export type UsageSnapshot = z.infer<typeof UsageSnapshotSchema>;

export function rateWindows(snapshot: UsageSnapshot): UsageWindow[] {
  return [
    snapshot.primary,
    ...(snapshot.secondary ? [snapshot.secondary] : []),
    ...(snapshot.tertiary ? [snapshot.tertiary] : []),
    ...snapshot.extra_windows,
  ];
}

export const UsageSnapshotsSchema = z.array(UsageSnapshotSchema);

export type UsageSnapshots = z.infer<typeof UsageSnapshotsSchema>;

export const ProviderUsageStateKindSchema = z.enum([
  "fresh",
  "fetching",
  "stale_error",
  "missing_credentials",
  "credentials_need_refresh",
  "error",
]);

export type ProviderUsageStateKind = z.infer<typeof ProviderUsageStateKindSchema>;

export const ProviderUsageStateSchema = z.object({
  provider: ProviderIdSchema,
  kind: ProviderUsageStateKindSchema,
  snapshot: UsageSnapshotSchema.nullable().optional(),
  health: ProviderHealthSchema,
  message: z.string().nullable().optional(),
  updated_at: z.string(),
});

export type ProviderUsageState = z.infer<typeof ProviderUsageStateSchema>;

export const ProviderUsageStatesSchema = z.array(ProviderUsageStateSchema);

export type ProviderUsageStates = z.infer<typeof ProviderUsageStatesSchema>;

const PROVIDER_ID_ALIASES: Record<string, ProviderId> = {
  opencodego: "opencode-go",
  "open-code-go": "opencode-go",
  "open-code": "opencode",
  droid: "factory",
};

export function normalizeProviderId(value: unknown): ProviderId | null {
  if (typeof value !== "string") {
    return null;
  }

  const normalized = PROVIDER_ID_ALIASES[value] ?? value;
  const parsed = ProviderIdSchema.safeParse(normalized);
  return parsed.success ? parsed.data : null;
}

function normalizeUsageSnapshotInput(snapshot: unknown): unknown {
  if (!snapshot || typeof snapshot !== "object" || !("provider" in snapshot)) {
    return snapshot;
  }

  const provider = normalizeProviderId((snapshot as { provider: unknown }).provider);
  if (!provider) {
    return snapshot;
  }

  return { ...snapshot, provider };
}

export function parseUsageSnapshots(data: unknown): UsageSnapshots {
  if (!Array.isArray(data)) {
    return UsageSnapshotsSchema.parse(data);
  }

  const normalized = data.map(normalizeUsageSnapshotInput).filter((snapshot) => {
    if (!snapshot || typeof snapshot !== "object" || !("provider" in snapshot)) {
      return false;
    }

    return ProviderIdSchema.safeParse((snapshot as { provider: unknown }).provider).success;
  });

  return UsageSnapshotsSchema.parse(normalized);
}

function normalizeProviderUsageStateInput(state: unknown): unknown {
  if (!state || typeof state !== "object" || !("provider" in state)) {
    return state;
  }

  const provider = normalizeProviderId((state as { provider: unknown }).provider);
  if (!provider) {
    return state;
  }

  const snapshot = "snapshot" in state ? normalizeUsageSnapshotInput(state.snapshot) : undefined;
  return { ...state, provider, snapshot };
}

export function parseProviderUsageStates(data: unknown): ProviderUsageStates {
  if (!Array.isArray(data)) {
    return ProviderUsageStatesSchema.parse(data);
  }

  const normalized = data.map(normalizeProviderUsageStateInput).filter((state) => {
    if (!state || typeof state !== "object" || !("provider" in state)) {
      return false;
    }

    return ProviderIdSchema.safeParse((state as { provider: unknown }).provider).success;
  });

  return ProviderUsageStatesSchema.parse(normalized);
}

export const UpdateInfoSchema = z.object({
  available: z.boolean(),
  version: z.string().nullable(),
  channel: z.string(),
  notes: z.string().nullable(),
});

export type UpdateInfo = z.infer<typeof UpdateInfoSchema>;

export const UpdateDownloadProgressSchema = z.object({
  downloaded: z.number().int().nonnegative(),
  total: z.number().int().nonnegative().nullable(),
});

export type UpdateDownloadProgress = z.infer<typeof UpdateDownloadProgressSchema>;
