import { useCallback, useState } from "react";

import { refreshAllProviders } from "@/lib/tauri/commands";

export function useTrayPanelRefresh() {
  const [isRefreshingAll, setIsRefreshingAll] = useState(false);

  const refreshAll = useCallback(async () => {
    if (isRefreshingAll) return;
    setIsRefreshingAll(true);
    try {
      await refreshAllProviders();
    } finally {
      setIsRefreshingAll(false);
    }
  }, [isRefreshingAll]);

  return { refreshAll, isRefreshingAll };
}
