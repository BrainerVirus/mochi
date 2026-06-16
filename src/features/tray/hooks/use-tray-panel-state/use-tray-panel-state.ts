import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";

import { useSettings } from "@/features/tray/hooks/use-tray-events";
import { useTrayPanelRefresh } from "@/features/tray/hooks/use-tray-panel-refresh";
import { useTrayUiStore } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { useUsageData } from "@/features/usage/hooks/use-usage-data/use-usage-data";
import { queryKeys } from "@/lib/query/keys";
import type { ProviderId } from "@/lib/schemas/usage";
import { refreshSingleProvider, saveSettings, syncTrayUsage } from "@/lib/tauri/commands";
import {
  buildTrayPanelTabsFromStates,
  filterUsageStatesForTrayPanel,
} from "@/lib/utils/tray-panel-tabs";
import { parseTrayTabChange } from "@/lib/utils/tray-tab-selection";

export function useTrayPanelState() {
  const { data: settings } = useSettings();
  const { data, error, isError, isPending, isSuccess, isFetching } = useUsageData();
  const selectedTab = useTrayUiStore((state) => state.selectedTab);
  const setSelectedTab = useTrayUiStore((state) => state.setSelectedTab);
  const [refreshingProvider, setRefreshingProvider] = useState<ProviderId | null>(null);
  const pendingRefreshes = useRef(0);

  const queryClient = useQueryClient();

  const enabledProviders = useMemo(
    () => settings?.enabled_providers ?? [],
    [settings?.enabled_providers],
  );
  const { refreshAll, isRefreshingAll } = useTrayPanelRefresh();

  const states = filterUsageStatesForTrayPanel(data ?? [], enabledProviders);
  const tabs = buildTrayPanelTabsFromStates(data ?? [], enabledProviders);

  useEffect(() => {
    if (tabs.some((tab) => tab.id === selectedTab)) {
      return;
    }
    setSelectedTab("overview");
  }, [selectedTab, setSelectedTab, tabs]);

  useEffect(() => {
    void syncTrayUsage(selectedTab);
  }, [selectedTab]);

  function handleTabChange(value: string) {
    const nextTab = parseTrayTabChange(value);
    setSelectedTab(nextTab);
    void syncTrayUsage(nextTab);

    // Persist to shared settings (both windows read same settings.json)
    if (settings) {
      const updated = { ...settings, selected_tab: nextTab };
      void saveSettings(updated).then(() => {
        queryClient.setQueryData(queryKeys.settings, updated);
      });
    }
  }

  function handleRefreshProvider(provider: ProviderId) {
    pendingRefreshes.current++;
    setRefreshingProvider(provider);
    void refreshSingleProvider(provider)
      .catch(() => {
        void queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots }).catch(() => {});
      })
      .finally(() => {
        pendingRefreshes.current--;
        if (pendingRefreshes.current === 0) {
          setRefreshingProvider(null);
        }
      });
  }

  return {
    settings,
    data,
    error,
    isError,
    isPending,
    isSuccess,
    isFetching,
    refreshAll,
    isRefreshingAll,
    selectedTab,
    refreshingProvider,
    states,
    tabs,
    handleTabChange,
    handleRefreshProvider,
  };
}
