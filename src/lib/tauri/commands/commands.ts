import { invoke } from "@tauri-apps/api/core";

import type { TraySelectedTab } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { detectPlatformFromNavigator, parsePlatformId } from "@/lib/platform/detect";
import type { PlatformId } from "@/lib/platform/types";
import {
  ProviderCatalogSchema,
  ProviderCredentialStatusSchema,
  type ProviderCatalogEntry,
  type ProviderCredentialDetail,
} from "@/lib/schemas/provider-catalog";
import {
  DEFAULT_MOCHI_SETTINGS,
  MochiSettingsSchema,
  type MochiSettings,
  type UpdateChannel,
} from "@/lib/schemas/settings";
import {
  ProviderUsageStatesSchema,
  UpdateInfoSchema,
  UsageSnapshotsSchema,
  parseProviderUsageStates,
  type ProviderId,
  type ProviderUsageStates,
  type UpdateInfo,
  type UsageSnapshots,
} from "@/lib/schemas/usage";
import { isTauriRuntime } from "@/lib/tauri/runtime";

export function appVersion(): Promise<string> {
  return invoke<string>("app_version");
}

export async function getPlatform(): Promise<PlatformId> {
  if (!isTauriRuntime()) {
    return detectPlatformFromNavigator();
  }

  const result = await invoke<string>("get_platform");
  return parsePlatformId(result);
}

export async function checkForUpdate(channel: string): Promise<UpdateInfo> {
  const result = await invoke<unknown>("check_for_update", { channel });
  return UpdateInfoSchema.parse(result);
}

export function installUpdate(channel: string): Promise<void> {
  return invoke<void>("install_update", { channel });
}

export async function getUsageStates(): Promise<ProviderUsageStates> {
  const result = await invoke<unknown>("get_usage_snapshots");
  return parseProviderUsageStates(result);
}

export async function refreshEnabledProviders(): Promise<UsageSnapshots> {
  const result = await invoke<unknown>("refresh_enabled_providers");
  return UsageSnapshotsSchema.parse(result);
}

export async function refreshAllProviders(): Promise<ProviderUsageStates> {
  const result = await invoke<{ states: unknown }>("refresh_all_providers");
  return ProviderUsageStatesSchema.parse(result.states);
}

export async function refreshSingleProvider(provider: ProviderId): Promise<ProviderUsageStates> {
  const result = await invoke<{ states: unknown }>("refresh_single_provider", { provider });
  return ProviderUsageStatesSchema.parse(result.states);
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

export function syncTrayUpdateChannel(channel: UpdateChannel): Promise<void> {
  return invoke<void>("sync_tray_update_channel", { channel });
}

export function showMainPanel(): Promise<void> {
  return invoke<void>("show_main_panel");
}

export function openAppWindow(path: string): Promise<void> {
  return invoke<void>("open_app_window", { path });
}

export async function getProviderCatalog(): Promise<ProviderCatalogEntry[]> {
  const result = await invoke<unknown>("get_provider_catalog");
  return ProviderCatalogSchema.parse(result);
}

export async function getProviderCredentialStatus(): Promise<
  Record<string, ProviderCredentialDetail>
> {
  const result = await invoke<unknown>("get_provider_credential_status");
  return ProviderCredentialStatusSchema.parse(result);
}

export function openExternalUrl(url: string): Promise<void> {
  return invoke<void>("open_external_url", { url });
}

export function setTrayPanelHeight(height: number): Promise<void> {
  return invoke<void>("set_tray_panel_height", { height });
}

export function setWidgetHeight(height: number): Promise<void> {
  return invoke<void>("set_widget_height", { height });
}

export function quitApp(): Promise<void> {
  if (typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window)) {
    return Promise.resolve();
  }

  return invoke<void>("quit_app");
}
