import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { useSettings } from "@/hooks/use-tray-events";
import { useUsageData } from "@/hooks/use-usage-data";
import { queryKeys } from "@/lib/query/keys";
import type { MochiSettings } from "@/lib/schemas/settings";
import type { ProviderUsageState } from "@/lib/schemas/usage";
import { syncCurrentTrayUsage } from "@/lib/stores/tray-ui-store";
import { refreshEnabledProviders } from "@/lib/tauri/commands";
import { isTauriRuntime } from "@/lib/tauri/runtime";

export function shouldRefreshEnabledProvidersOnBoot(
  settings: MochiSettings,
  states: ProviderUsageState[],
): boolean {
  if (settings.enabled_providers.length === 0) {
    return false;
  }

  const enabled = new Set(settings.enabled_providers);
  return states.some((state) => enabled.has(state.provider) && state.kind === "fetching");
}

export async function runColdStartProviderRefreshSequence(
  settings: MochiSettings,
  refresh: () => Promise<unknown>,
  invalidate: () => Promise<unknown>,
  syncUsage: (settings: Pick<MochiSettings, "enabled_providers">) => Promise<unknown>,
) {
  await refresh();
  await invalidate();
  await syncUsage(settings);
}

export function useColdStartProviderRefresh() {
  const queryClient = useQueryClient();
  const { data: settings } = useSettings();
  const { data: states = [] } = useUsageData();
  const didRefreshRef = useRef(false);

  useEffect(() => {
    if (
      didRefreshRef.current ||
      !isTauriRuntime() ||
      !settings ||
      !shouldRefreshEnabledProvidersOnBoot(settings, states)
    ) {
      return;
    }

    didRefreshRef.current = true;
    void runColdStartProviderRefreshSequence(
      settings,
      refreshEnabledProviders,
      () => queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots }),
      syncCurrentTrayUsage,
    ).catch(() => {
      didRefreshRef.current = false;
    });
  }, [queryClient, settings, states]);
}
