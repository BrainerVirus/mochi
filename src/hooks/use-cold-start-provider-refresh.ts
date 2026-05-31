import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { useSettings } from "@/hooks/use-tray-events";
import { useUsageData } from "@/hooks/use-usage-data";
import { queryKeys } from "@/lib/query/keys";
import type { MochiSettings } from "@/lib/schemas/settings";
import type { UsageSnapshot } from "@/lib/schemas/usage";
import { refreshEnabledProviders, syncTrayUsage } from "@/lib/tauri/commands";
import { isTauriRuntime } from "@/lib/tauri/runtime";

export function shouldRefreshEnabledProvidersOnBoot(
  settings: MochiSettings,
  snapshots: UsageSnapshot[],
): boolean {
  if (settings.enabled_providers.length === 0) {
    return false;
  }

  const enabled = new Set(settings.enabled_providers);
  return snapshots.some(
    (snapshot) => enabled.has(snapshot.provider) && snapshot.source === "credentials-detected",
  );
}

export function useColdStartProviderRefresh() {
  const queryClient = useQueryClient();
  const { data: settings } = useSettings();
  const { data: snapshots = [] } = useUsageData();
  const didRefreshRef = useRef(false);

  useEffect(() => {
    if (
      didRefreshRef.current ||
      !isTauriRuntime() ||
      !settings ||
      !shouldRefreshEnabledProvidersOnBoot(settings, snapshots)
    ) {
      return;
    }

    didRefreshRef.current = true;
    void refreshEnabledProviders()
      .then((nextSnapshots) => {
        queryClient.setQueryData(queryKeys.usageSnapshots, nextSnapshots);
        void syncTrayUsage();
      })
      .catch(() => {
        didRefreshRef.current = false;
      });
  }, [queryClient, settings, snapshots]);
}
