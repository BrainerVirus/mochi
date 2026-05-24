import { UpdateCheckPrefetch } from "@/components/updates/update-check-prefetch";
import { usePostUpdateRefresh } from "@/hooks/use-post-update-refresh";
import { useTrayEvents } from "@/hooks/use-tray-events";
import { useTrayUsageSync } from "@/hooks/use-tray-usage-sync";

export function TrayEventBridge() {
  useTrayEvents();
  useTrayUsageSync();
  usePostUpdateRefresh();

  return <UpdateCheckPrefetch />;
}
