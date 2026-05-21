import { invoke } from "@tauri-apps/api/core";

import { MochiSettingsSchema, type MochiSettings } from "@/lib/schemas/settings";
import {
  UpdateInfoSchema,
  UsageSnapshotSchema,
  UsageSnapshotsSchema,
  type ProviderId,
  type UpdateInfo,
  type UsageSnapshot,
  type UsageSnapshots,
} from "@/lib/schemas/usage";

export function appVersion(): Promise<string> {
  return invoke<string>("app_version");
}

export async function checkForUpdate(channel: string): Promise<UpdateInfo> {
  const result = await invoke<unknown>("check_for_update", { channel });
  return UpdateInfoSchema.parse(result);
}

export function installUpdate(): Promise<void> {
  return invoke<void>("install_update");
}

export async function getUsageSnapshots(): Promise<UsageSnapshots> {
  const result = await invoke<unknown>("get_usage_snapshots");
  return UsageSnapshotsSchema.parse(result);
}

export async function refreshProvider(provider: ProviderId): Promise<UsageSnapshot> {
  const result = await invoke<unknown>("refresh_provider", { provider });
  return UsageSnapshotSchema.parse(result);
}

export async function getSettings(): Promise<MochiSettings> {
  const result = await invoke<unknown>("get_settings");
  return MochiSettingsSchema.parse(result);
}

export async function saveSettings(settings: MochiSettings): Promise<MochiSettings> {
  const result = await invoke<unknown>("save_settings", { settings });
  return MochiSettingsSchema.parse(result);
}
