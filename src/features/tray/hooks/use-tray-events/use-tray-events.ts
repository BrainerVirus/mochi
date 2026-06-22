import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

import {
  syncCurrentTrayUsage,
  useTrayUiStore,
} from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { queryKeys } from "@/lib/query/keys";
import { saveSettingsMutationOptions, settingsQueryOptions } from "@/lib/query/settings";
import {
  MochiSettingsSchema,
  type MochiSettings,
  type UpdateChannel,
} from "@/lib/schemas/settings";
import { ProviderUsageStatesSchema, type ProviderUsageState } from "@/lib/schemas/usage";
import { openAppWindow, saveSettings, syncTrayUpdateChannel } from "@/lib/tauri/commands";
import { logFrontendDebug, reportFrontendError } from "@/lib/tauri/diagnostics";
import {
  shouldHandleAppNavigateEvent,
  shouldHandleTrayNavigateEvent,
} from "@/lib/tauri/window-events";
import { filterUsageStatesForTrayPanel } from "@/lib/utils/tray-panel-tabs";
import { parseTrayTabChange } from "@/lib/utils/tray-tab-selection";

export function useSettings() {
  return useQuery(settingsQueryOptions);
}

export function useSaveSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    ...saveSettingsMutationOptions(),
    onSuccess: (settings) => {
      // Reconcile immediately in the saving webview. Other webviews rely on settings-changed.
      handleSettingsSaveSuccess(queryClient, settings);
    },
  });
}

export function cacheSavedSettings(
  queryClient: Pick<SettingsSaveSuccessQueryClient, "setQueryData">,
  settings: MochiSettings,
): void {
  queryClient.setQueryData(queryKeys.settings, settings);
}

export function handleSettingsSaveSuccess(
  queryClient: SettingsSaveSuccessQueryClient,
  settings: MochiSettings,
  reconcile: (
    queryClient: SettingsSaveSuccessQueryClient,
    settings: MochiSettings,
  ) => Promise<void> = reconcileSettingsSaveSuccess,
): void {
  cacheSavedSettings(queryClient, settings);
  void reconcile(queryClient, settings).catch(logSettingsReconcileFailure);
}

export function useTrayEvents() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  useEffect(() => {
    const unlistenPromises = [
      listen<string>("tray-navigate", (event) => {
        if (!shouldHandleTrayNavigateEvent()) {
          return;
        }

        void navigate({ to: event.payload });
      }),
      listen<{ states: ProviderUsageState[] }>("usage-refresh-complete", (event) => {
        const result = ProviderUsageStatesSchema.safeParse(event.payload.states);
        if (!result.success) return;
        handleUsageRefreshComplete(
          result.data,
          (key, data) => queryClient.setQueryData(key, data),
          () =>
            queryClient.getQueryData<Pick<MochiSettings, "enabled_providers">>(queryKeys.settings),
        );
      }),
      listen<UpdateChannel>("tray-set-channel", (event) => {
        handleTraySetChannelEvent(event.payload, queryClient);
      }),
      listen("tray-check-update", () => {
        void openAppWindow("/update");
      }),
      listen<string>("set-tab", (event) => {
        handleSetTabEvent(event.payload);
      }),
      listen<string>("app-navigate", (event) => {
        if (!shouldHandleAppNavigateEvent()) {
          return;
        }

        void navigate({ to: event.payload });
      }),
      listen<unknown>("settings-changed", (event) => {
        handleSettingsChangedEvent(event.payload, queryClient);
      }),
    ];

    return () => {
      void Promise.all(unlistenPromises).then((unlisteners) => {
        for (const unlisten of unlisteners) {
          unlisten();
        }
      });
    };
  }, [navigate, queryClient]);
}

export function handleUsageRefreshComplete(
  states: ProviderUsageState[],
  setQueryData: (queryKey: readonly unknown[], data: ProviderUsageState[]) => unknown,
  getSettings: () => Pick<MochiSettings, "enabled_providers"> | undefined,
): void {
  setQueryData(queryKeys.usageSnapshots, states);
  const settings = getSettings();
  if (settings) {
    void syncCurrentTrayUsage(settings).catch(() => {
      // Tray icon sync failure is non-fatal; cache is already updated
    });
  }
}

export function parseSettingsChangedPayload(payload: unknown): MochiSettings | null {
  const result = MochiSettingsSchema.safeParse(payload);
  return result.success ? result.data : null;
}

function logInvalidSettingsChangedPayload(message: string): void {
  void reportFrontendError(message, "settings-changed");
}

function logSettingsReconcileFailure(error: unknown): void {
  const message = error instanceof Error ? error.message : String(error);
  logFrontendDebug("settings-reconcile", message);
  void reportFrontendError(message, "debug:settings-reconcile");
}

function providerListsMatch(
  left: MochiSettings["enabled_providers"],
  right: MochiSettings["enabled_providers"],
): boolean {
  return left.length === right.length && left.every((provider, index) => provider === right[index]);
}

function providerConfigsMatch(
  left: MochiSettings["provider_configs"],
  right: MochiSettings["provider_configs"],
): boolean {
  const leftKeys = Object.keys(left).toSorted();
  const rightKeys = Object.keys(right).toSorted();

  if (leftKeys.length !== rightKeys.length) {
    return false;
  }

  return leftKeys.every(
    (key, index) =>
      key === rightKeys[index] && JSON.stringify(left[key]) === JSON.stringify(right[key]),
  );
}

export function settingsCacheMatches(
  cached: MochiSettings | undefined,
  next: MochiSettings,
): boolean {
  if (!cached) {
    return false;
  }

  return (
    cached.update_channel === next.update_channel &&
    cached.refresh_interval_seconds === next.refresh_interval_seconds &&
    cached.show_notifications === next.show_notifications &&
    cached.selected_tab === next.selected_tab &&
    providerListsMatch(cached.enabled_providers, next.enabled_providers) &&
    providerConfigsMatch(cached.provider_configs, next.provider_configs)
  );
}

function readSettingsCache(queryClient: SettingsSaveSuccessQueryClient): MochiSettings | undefined {
  const result = MochiSettingsSchema.safeParse(queryClient.getQueryData(queryKeys.settings));
  return result.success ? result.data : undefined;
}

function readUsageCache(
  queryClient: SettingsSaveSuccessQueryClient,
): ProviderUsageState[] | undefined {
  const result = ProviderUsageStatesSchema.safeParse(
    queryClient.getQueryData(queryKeys.usageSnapshots),
  );
  return result.success ? result.data : undefined;
}

export function handleSettingsChangedEvent(
  payload: unknown,
  queryClient: SettingsSaveSuccessQueryClient,
  reconcile: (
    queryClient: SettingsSaveSuccessQueryClient,
    settings: MochiSettings,
  ) => Promise<void> = reconcileSettingsSaveSuccess,
  logInvalid: (message: string) => void = logInvalidSettingsChangedPayload,
): void {
  const settings = parseSettingsChangedPayload(payload);
  if (!settings) {
    logInvalid("settings-changed payload failed validation");
    return;
  }

  if (settingsCacheMatches(readSettingsCache(queryClient), settings)) {
    return;
  }

  void reconcile(queryClient, settings).catch(logSettingsReconcileFailure);
}

export function handleTraySetChannelEvent(
  channel: UpdateChannel,
  queryClient: SettingsSaveSuccessQueryClient,
  save: (settings: MochiSettings) => Promise<MochiSettings> = saveSettings,
): void {
  const current = readSettingsCache(queryClient);
  if (!current) {
    return;
  }

  void save({
    ...current,
    update_channel: channel,
  }).then((settings) => {
    handleSettingsSaveSuccess(queryClient, settings);
  });
}

export function handleSetTabEvent(payload: string): void {
  const tab = parseTrayTabChange(payload);
  useTrayUiStore.getState().setSelectedTab(tab);
}

export async function reconcileSettingsSaveSuccess(
  queryClient: SettingsSaveSuccessQueryClient,
  settings: MochiSettings,
  syncChannel: (channel: UpdateChannel) => Promise<void> = syncTrayUpdateChannel,
): Promise<void> {
  queryClient.setQueryData(queryKeys.settings, settings);
  pruneDisabledProvidersFromUsageCache(queryClient, settings.enabled_providers);
  await syncChannel(settings.update_channel);
  await queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
}

function pruneDisabledProvidersFromUsageCache(
  queryClient: SettingsSaveSuccessQueryClient,
  enabledProviders: MochiSettings["enabled_providers"],
): void {
  const cached = readUsageCache(queryClient);
  if (!cached) {
    return;
  }

  queryClient.setQueryData(
    queryKeys.usageSnapshots,
    enabledProviders.length === 0 ? [] : filterUsageStatesForTrayPanel(cached, enabledProviders),
  );
}

interface SettingsSaveSuccessQueryClient {
  setQueryData: (queryKey: readonly unknown[], data: unknown) => unknown;
  getQueryData: (queryKey: readonly unknown[]) => unknown;
  invalidateQueries: (options: { queryKey: readonly unknown[] }) => Promise<unknown>;
}
