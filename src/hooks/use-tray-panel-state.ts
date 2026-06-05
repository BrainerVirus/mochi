import { useEffect, useMemo, useState } from "react";

import { useRefreshProvider, useSettings } from "@/hooks/use-tray-events";
import { useTrayPanelRefresh } from "@/hooks/use-tray-panel-refresh";
import { useUsageData } from "@/hooks/use-usage-data";
import type { ProviderId } from "@/lib/schemas/usage";
import { useTrayUiStore } from "@/lib/stores/tray-ui-store";
import { syncTrayUsage } from "@/lib/tauri/commands";
import {
  buildTrayPanelTabsFromStates,
  filterUsageStatesForTrayPanel,
} from "@/lib/utils/tray-panel-tabs";
import { parseTrayTabChange } from "@/lib/utils/tray-tab-selection";

export function useTrayPanelState() {
  const { data: settings } = useSettings();
  const { data, error, isError, isPending, isSuccess, refetch, isFetching } = useUsageData();
  const refreshProviderMutation = useRefreshProvider();
  const selectedTab = useTrayUiStore((state) => state.selectedTab);
  const setSelectedTab = useTrayUiStore((state) => state.setSelectedTab);
  const [refreshingProvider, setRefreshingProvider] = useState<ProviderId | null>(null);

  const enabledProviders = useMemo(
    () => settings?.enabled_providers ?? [],
    [settings?.enabled_providers],
  );
  const { refreshAll, isRefreshingAll } = useTrayPanelRefresh({
    enabledProviders,
    refetch: () => refetch(),
    selectedTab,
  });

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
  }

  function handleRefreshProvider(provider: ProviderId) {
    setRefreshingProvider(provider);
    refreshProviderMutation.mutate(provider, {
      onSettled: () => {
        setRefreshingProvider(null);
        void syncTrayUsage(selectedTab);
      },
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
    refreshProviderMutation,
    selectedTab,
    refreshingProvider,
    states,
    tabs,
    handleTabChange,
    handleRefreshProvider,
  };
}
