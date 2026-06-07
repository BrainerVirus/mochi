import { QueryClientProvider } from "@tanstack/react-query";
import { Outlet } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import {
  getHydrationSafeRootState,
  shouldUseFullHeightWindowShell,
} from "@/components/layout/root-component-state";
import { TrayEventBridge } from "@/features/tray/components/tray-event-bridge";
import { detectPlatform } from "@/lib/platform/detect";
import { useSystemColorScheme } from "@/lib/platform/use-system-color-scheme";
import { queryClient } from "@/lib/query/client";
import { readIsAppWindow } from "@/lib/tauri/app-window";
import { readIsTrayPanelWindow } from "@/lib/tauri/tray-panel-window";
import { readIsWidgetWindow } from "@/lib/tauri/widget-window";

export function RootComponent() {
  const [rootState, setRootState] = useState(getHydrationSafeRootState);
  const { isTrayPanelWindow, isAppWindow, isWidgetWindow, platform } = rootState;
  const isFullHeightWindowShell = shouldUseFullHeightWindowShell({
    isTrayPanelWindow,
    isAppWindow,
    isWidgetWindow,
    platform,
  });
  const supportsNativeWindowGlass = platform === "macos" || platform === "windows";
  const isNativeGlassShell = supportsNativeWindowGlass && (isTrayPanelWindow || isAppWindow);

  useSystemColorScheme(!isNativeGlassShell);

  useEffect(() => {
    const isTrayWindow = readIsTrayPanelWindow();
    const isDedicatedAppWindow = readIsAppWindow();
    const isWidget = readIsWidgetWindow();
    void detectPlatform().then((detectedPlatform) => {
      setRootState({
        isTrayPanelWindow: isTrayWindow,
        isAppWindow: isDedicatedAppWindow,
        isWidgetWindow: isWidget,
        platform: detectedPlatform,
      });
    });
  }, []);

  useEffect(() => {
    document.documentElement.dataset.platform = platform;
    document.documentElement.toggleAttribute("data-tray-panel", isTrayPanelWindow);
    document.documentElement.toggleAttribute("data-app-window", isAppWindow);
    document.documentElement.toggleAttribute("data-widget-window", isWidgetWindow);
    document.documentElement.classList.toggle("h-full", isFullHeightWindowShell);
    document.documentElement.classList.toggle("bg-transparent", isNativeGlassShell);
    document.body.className = isFullHeightWindowShell
      ? `flex h-full min-h-0 flex-1 flex-col overflow-hidden ${
          isNativeGlassShell ? "bg-transparent" : ""
        }`.trim()
      : "";
  }, [
    isAppWindow,
    isFullHeightWindowShell,
    isNativeGlassShell,
    isTrayPanelWindow,
    isWidgetWindow,
    platform,
  ]);

  return (
    <QueryClientProvider client={queryClient}>
      <div
        data-platform={platform}
        data-tray-panel={isTrayPanelWindow ? "" : undefined}
        data-app-window={isAppWindow ? "" : undefined}
        data-widget-window={isWidgetWindow ? "" : undefined}
        className={
          isFullHeightWindowShell
            ? "flex h-full min-h-0 flex-1 flex-col overflow-hidden"
            : undefined
        }
      >
        <TrayEventBridge />
        <Outlet />
      </div>
    </QueryClientProvider>
  );
}
