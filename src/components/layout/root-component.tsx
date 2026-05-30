import { QueryClientProvider } from "@tanstack/react-query";
import { Outlet } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { getHydrationSafeRootState } from "@/components/layout/root-component-state";
import { TrayEventBridge } from "@/components/tray/tray-event-bridge";
import { detectPlatform, useSystemColorScheme } from "@/lib/platform";
import { queryClient } from "@/lib/query/client";
import { readIsAppWindow } from "@/lib/tauri/app-window";
import { readIsTrayPanelWindow } from "@/lib/tauri/tray-panel-window";

export function RootComponent() {
  const [rootState, setRootState] = useState(getHydrationSafeRootState);
  const { isTrayPanelWindow, isAppWindow, platform } = rootState;
  const supportsNativeWindowGlass = platform === "macos" || platform === "windows";
  const isNativeGlassShell = supportsNativeWindowGlass && (isTrayPanelWindow || isAppWindow);

  useSystemColorScheme(!isNativeGlassShell);

  useEffect(() => {
    const isTrayWindow = readIsTrayPanelWindow();
    const isDedicatedAppWindow = readIsAppWindow();
    void detectPlatform().then((detectedPlatform) => {
      setRootState({
        isTrayPanelWindow: isTrayWindow,
        isAppWindow: isDedicatedAppWindow,
        platform: detectedPlatform,
      });
    });
  }, []);

  useEffect(() => {
    document.documentElement.dataset.platform = platform;
    document.documentElement.toggleAttribute("data-tray-panel", isTrayPanelWindow);
    document.documentElement.toggleAttribute("data-app-window", isAppWindow);
    document.documentElement.classList.toggle("h-full", isNativeGlassShell);
    document.documentElement.classList.toggle("bg-transparent", isNativeGlassShell);
    document.body.className = isNativeGlassShell
      ? "flex h-full min-h-0 flex-1 flex-col overflow-hidden bg-transparent"
      : "";
  }, [isAppWindow, isNativeGlassShell, isTrayPanelWindow, platform]);

  return (
    <QueryClientProvider client={queryClient}>
      <div
        data-platform={platform}
        data-tray-panel={isTrayPanelWindow ? "" : undefined}
        data-app-window={isAppWindow ? "" : undefined}
        className={
          isNativeGlassShell ? "flex h-full min-h-0 flex-1 flex-col overflow-hidden" : undefined
        }
      >
        <TrayEventBridge />
        <Outlet />
      </div>
    </QueryClientProvider>
  );
}
