import { UpdateCheckPrefetch } from "@/components/updates/update-check-prefetch";
import { useColdStartProviderRefresh } from "@/hooks/use-cold-start-provider-refresh";
import { useDiagnosticsBoot } from "@/hooks/use-diagnostics-boot";
import { useInitialWindowRoute } from "@/hooks/use-initial-window-route";
import { usePostUpdateRefresh } from "@/hooks/use-post-update-refresh";
import { useTrayEvents } from "@/hooks/use-tray-events";
import { useTrayUsageSync } from "@/hooks/use-tray-usage-sync";

export function TrayEventBridge() {
  useDiagnosticsBoot();
  useInitialWindowRoute();
  useTrayEvents();
  useTrayUsageSync();
  usePostUpdateRefresh();
  useColdStartProviderRefresh();

  return <UpdateCheckPrefetch />;
}
