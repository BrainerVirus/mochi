import { invoke } from "@tauri-apps/api/core";

import {
  DEFAULT_MOCHI_SETTINGS,
  MochiSettingsSchema,
  type MochiSettings,
} from "@/lib/schemas/settings";
import { isTauriRuntime } from "@/lib/tauri/runtime";
import {
  UpdateInfoSchema,
  UsageSnapshotSchema,
  UsageSnapshotsSchema,
  type ProviderId,
  type UpdateInfo,
  type UsageSnapshot,
  type UsageSnapshots,
} from "@/lib/schemas/usage";
import type { TraySelectedTab } from "@/lib/stores/tray-ui-store";

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
  if (!isTauriRuntime()) {
    return DEFAULT_MOCHI_SETTINGS;
  }

  const result = await invoke<unknown>("get_settings");
  return MochiSettingsSchema.parse(result);
}

export async function saveSettings(settings: MochiSettings): Promise<MochiSettings> {
  const result = await invoke<unknown>("save_settings", { settings });
  return MochiSettingsSchema.parse(result);
}

export function showWidget(): Promise<void> {
  return invoke<void>("show_widget");
}

export function hideWidget(): Promise<void> {
  return invoke<void>("hide_widget");
}

export function toggleWidget(): Promise<void> {
  return invoke<void>("toggle_widget");
}

export function syncTrayUsage(selection?: TraySelectedTab): Promise<void> {
  return invoke<void>("sync_tray_usage", { selection: selection ?? null });
}

export function showMainPanel(): Promise<void> {
  return invoke<void>("show_main_panel");
}

export function openAppWindow(path: string): Promise<void> {
  return invoke<void>("open_app_window", { path });
}

export function openExternalUrl(url: string): Promise<void> {
  return invoke<void>("open_external_url", { url });
}

export function setTrayPanelHeight(height: number): Promise<void> {
  return invoke<void>("set_tray_panel_height", { height });
}

export function quitApp(): Promise<void> {
  if (typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window)) {
    return Promise.resolve();
  }

  return invoke<void>("quit_app");
}
