import { useTrayEvents } from "@/features/tray/hooks/use-tray-events";
import { useTrayUsageSync } from "@/features/tray/hooks/use-tray-usage-sync";
import { UpdateCheckPrefetch } from "@/features/updates/components/update-check-prefetch";
import { usePostUpdateRefresh } from "@/features/updates/hooks/use-post-update-refresh/use-post-update-refresh";
import { useColdStartProviderRefresh } from "@/hooks/use-cold-start-provider-refresh";
import { useDiagnosticsBoot } from "@/hooks/use-diagnostics-boot";
import { useInitialWindowRoute } from "@/hooks/use-initial-window-route";

export function TrayEventBridge() {
  useDiagnosticsBoot();
  useInitialWindowRoute();
  useTrayEvents();
  useTrayUsageSync();
  usePostUpdateRefresh();
  useColdStartProviderRefresh();

  return <UpdateCheckPrefetch />;
}
