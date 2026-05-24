import type { PlatformId } from "@/lib/platform";

export interface HydrationSafeRootState {
  isTrayPanelWindow: boolean;
  platform: PlatformId;
}

export function getHydrationSafeRootState(): HydrationSafeRootState {
  return {
    isTrayPanelWindow: false,
    platform: "unknown",
  };
}
