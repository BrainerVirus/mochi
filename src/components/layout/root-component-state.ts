import type { PlatformId } from "@/lib/platform";

export interface HydrationSafeRootState {
  isTrayPanelWindow: boolean;
  isAppWindow: boolean;
  isWidgetWindow: boolean;
  platform: PlatformId;
}

export function getHydrationSafeRootState(): HydrationSafeRootState {
  return {
    isTrayPanelWindow: false,
    isAppWindow: false,
    isWidgetWindow: false,
    platform: "unknown",
  };
}

export function shouldUseFullHeightWindowShell({
  isTrayPanelWindow,
  isAppWindow,
  isWidgetWindow = false,
}: {
  isTrayPanelWindow: boolean;
  isAppWindow: boolean;
  isWidgetWindow?: boolean;
  platform: PlatformId;
}): boolean {
  return isTrayPanelWindow || isAppWindow || isWidgetWindow;
}
