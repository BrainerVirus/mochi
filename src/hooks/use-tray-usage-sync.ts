import { useEffect } from "react";

import { useUsageData } from "@/hooks/use-usage-data";
import { useTrayUiStore } from "@/lib/stores/tray-ui-store";
import { syncTrayUsage } from "@/lib/tauri/commands";

export function useTrayUsageSync() {
  const { data, isSuccess } = useUsageData();
  const selectedTab = useTrayUiStore((state) => state.selectedTab);

  // Tab changes and tray open: update menu bar from Rust cache immediately (no live fetch).
  useEffect(() => {
    void syncTrayUsage(selectedTab);
  }, [selectedTab]);

  useEffect(() => {
    if (!isSuccess) {
      return;
    }

    void syncTrayUsage(selectedTab);
  }, [data, isSuccess, selectedTab]);
}
