import { UpdateCheckPrefetch } from "@/components/updates/update-check-prefetch";
import { useTrayEvents } from "@/features/tray/hooks/use-tray-events";
import { useTrayUsageSync } from "@/features/tray/hooks/use-tray-usage-sync";
import { useColdStartProviderRefresh } from "@/hooks/use-cold-start-provider-refresh";
import { useDiagnosticsBoot } from "@/hooks/use-diagnostics-boot";
import { useInitialWindowRoute } from "@/hooks/use-initial-window-route";
import { usePostUpdateRefresh } from "@/hooks/use-post-update-refresh";

export function TrayEventBridge() {
  useDiagnosticsBoot();
  useInitialWindowRoute();
  useTrayEvents();
  useTrayUsageSync();
  usePostUpdateRefresh();
  useColdStartProviderRefresh();

  return <UpdateCheckPrefetch />;
}
