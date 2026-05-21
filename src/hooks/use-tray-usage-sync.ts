import { useEffect } from "react";

import { useUsageData } from "@/hooks/use-usage-data";
import { syncTrayUsage } from "@/lib/tauri/commands";
import { useTrayUiStore } from "@/lib/stores/tray-ui-store";

export function useTrayUsageSync() {
  const { data, isSuccess } = useUsageData();
  const selectedTab = useTrayUiStore((state) => state.selectedTab);

  useEffect(() => {
    if (!isSuccess) {
      return;
    }

    void syncTrayUsage(selectedTab);
  }, [data, isSuccess, selectedTab]);
}
