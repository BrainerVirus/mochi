import { useTrayEvents } from "@/hooks/use-tray-events";
import { useTrayUsageSync } from "@/hooks/use-tray-usage-sync";

export function TrayEventBridge() {
  useTrayEvents();
  useTrayUsageSync();
  return null;
}
