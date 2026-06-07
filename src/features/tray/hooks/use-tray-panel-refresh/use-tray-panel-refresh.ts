import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";

import type { TraySelectedTab } from "@/features/tray/lib/stores/tray-ui-store/tray-ui-store";
import { queryKeys } from "@/lib/query/keys";
import type { ProviderId } from "@/lib/schemas/usage";
import { refreshEnabledProviders, syncTrayUsage } from "@/lib/tauri/commands";

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
        await refreshEnabledProviders();
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
