import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";

import { queryKeys } from "@/lib/query/keys";
import type { ProviderId } from "@/lib/schemas/usage";
import type { TraySelectedTab } from "@/lib/stores/tray-ui-store";
import { refreshProvider, syncTrayUsage } from "@/lib/tauri/commands";

interface UseTrayPanelRefreshOptions {
  enabledProviders: ProviderId[];
  refetch: () => Promise<unknown>;
  selectedTab: TraySelectedTab;
}

export function useTrayPanelRefresh({
  enabledProviders,
  refetch,
  selectedTab,
}: UseTrayPanelRefreshOptions) {
  const queryClient = useQueryClient();
  const [isRefreshingAll, setIsRefreshingAll] = useState(false);

  const refreshAll = useCallback(async () => {
    if (isRefreshingAll) {
      return;
    }

    setIsRefreshingAll(true);
    try {
      if (enabledProviders.length > 0) {
        await Promise.allSettled(enabledProviders.map((provider) => refreshProvider(provider)));
      }
      await queryClient.invalidateQueries({ queryKey: queryKeys.usageSnapshots });
      await syncTrayUsage(selectedTab);
      await refetch();
    } finally {
      setIsRefreshingAll(false);
    }
  }, [enabledProviders, isRefreshingAll, queryClient, refetch, selectedTab]);

  return {
    refreshAll,
    isRefreshingAll,
  };
}
