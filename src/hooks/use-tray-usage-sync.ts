import { useEffect } from "react";

import { useUsageData } from "@/hooks/use-usage-data";
import { useTrayUiStore } from "@/lib/stores/tray-ui-store";
import { syncTrayUsage } from "@/lib/tauri/commands";

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
