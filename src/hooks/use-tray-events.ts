import {
  type QueryClient,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { queryKeys } from "@/lib/query/keys";
import { refreshProviderMutationOptions } from "@/lib/query/refresh-provider";
import { saveSettingsMutationOptions, settingsQueryOptions } from "@/lib/query/settings";
import type { MochiSettings, UpdateChannel } from "@/lib/schemas/settings";
import {
  openAppWindow,
  refreshEnabledProviders,
  saveSettings,
  syncTrayUsage,
} from "@/lib/tauri/commands";
import {
  shouldHandleAppNavigateEvent,
  shouldHandleTrayNavigateEvent,
} from "@/lib/tauri/window-events";

export function useSettings() {
  return useQuery(settingsQueryOptions);
}

export function useSaveSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    ...saveSettingsMutationOptions(),
    onSuccess: (settings) => {
      void reconcileSettingsSaveSuccess(queryClient, settings);
    },
  });
}

export function useRefreshProvider() {
  const queryClient = useQueryClient();

  return useMutation({
    ...refreshProviderMutationOptions(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
    },
  });
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
      listen("tray-refresh", () => {
        void refreshEnabledProviders()
          .catch(() => undefined)
          .then(() => queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots }))
          .then(() => syncTrayUsage());
      }),
      listen<UpdateChannel>("tray-set-channel", (event) => {
        const current = queryClient.getQueryData<MochiSettings>(queryKeys.settings);
        if (!current) {
          return;
        }

        void saveSettings({
          ...current,
          update_channel: event.payload,
        }).then((settings) => {
          void reconcileSettingsSaveSuccess(queryClient, settings);
        });
      }),
      listen("tray-check-update", () => {
        void openAppWindow("/update");
      }),
      listen<string>("app-navigate", (event) => {
        if (!shouldHandleAppNavigateEvent()) {
          return;
        }

        void navigate({ to: event.payload });
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

export function shouldRunProviderRefreshForTrayEvent(eventName: string): boolean {
  return eventName === "tray-refresh";
}

export async function reconcileSettingsSaveSuccess(
  queryClient: Pick<QueryClient, "setQueryData" | "invalidateQueries">,
  settings: MochiSettings,
  syncUsage: () => Promise<void> = syncTrayUsage,
): Promise<void> {
  queryClient.setQueryData(queryKeys.settings, settings);
  await queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
  await syncUsage();
}
