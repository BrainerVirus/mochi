import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { useSettings } from "@/features/tray/hooks/use-tray-events";
import { syncCurrentTrayUsage } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { useUsageData } from "@/features/usage/hooks/use-usage-data/use-usage-data";
import { queryKeys } from "@/lib/query/keys";
import type { MochiSettings } from "@/lib/schemas/settings";
import type { ProviderUsageState } from "@/lib/schemas/usage";
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
  const refreshIntervalSeconds = settings.refresh_interval_seconds;

  return states.some((state) => {
    if (!enabled.has(state.provider)) return false;

    if (state.kind === "fetching") return true;
    if (state.kind === "stale_error") return true;
    if (state.kind === "error") return true;

    if (state.kind === "fresh" && state.updated_at) {
      const ageMs = Date.now() - Date.parse(state.updated_at);
      const thresholdMs = refreshIntervalSeconds * 1000;
      return !Number.isFinite(ageMs) || ageMs < 0 || ageMs > thresholdMs;
    }

    return false;
  });
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
