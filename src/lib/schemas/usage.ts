import { z } from "zod";

export const ProviderIdSchema = z.enum([
  "codex",
  "claude",
  "cursor",
  "gemini",
  "copilot",
  "antigravity",
  "factory",
  "zai",
  "kiro",
  "augment",
]);

export type ProviderId = z.infer<typeof ProviderIdSchema>;

export const UsageWindowSchema = z.object({
  label: z.string(),
  used_percent: z.number(),
  remaining_percent: z.number(),
  resets_at: z.string().nullable(),
});

export type UsageWindow = z.infer<typeof UsageWindowSchema>;

export const UsageSnapshotSchema = z.object({
  provider: ProviderIdSchema,
  primary: UsageWindowSchema,
  secondary: UsageWindowSchema.nullable(),
  updated_at: z.string(),
  source: z.string(),
});

export type UsageSnapshot = z.infer<typeof UsageSnapshotSchema>;

export const UsageSnapshotsSchema = z.array(UsageSnapshotSchema);

export type UsageSnapshots = z.infer<typeof UsageSnapshotsSchema>;

export function parseUsageSnapshots(data: unknown): UsageSnapshots {
  return UsageSnapshotsSchema.parse(data);
}

export const UpdateInfoSchema = z.object({
  available: z.boolean(),
  version: z.string().nullable(),
  channel: z.string(),
  notes: z.string().nullable(),
});

export type UpdateInfo = z.infer<typeof UpdateInfoSchema>;
