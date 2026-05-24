import type { PlatformId } from "@/lib/platform";

export interface HydrationSafeRootState {
  isTrayPanelWindow: boolean;
  isAppWindow: boolean;
  platform: PlatformId;
}

export function getHydrationSafeRootState(): HydrationSafeRootState {
  return {
    isTrayPanelWindow: false,
    isAppWindow: false,
    platform: "unknown",
  };
}
