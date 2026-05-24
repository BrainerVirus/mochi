import { QueryClientProvider } from "@tanstack/react-query";
import { HeadContent, Outlet, Scripts } from "@tanstack/react-router";
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
  const isNativeGlassShell = isTrayPanelWindow || isAppWindow;

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

  return (
    <html
      lang="en"
      data-platform={platform}
      data-tray-panel={isTrayPanelWindow ? "" : undefined}
      data-app-window={isAppWindow ? "" : undefined}
      className={isNativeGlassShell ? "h-full bg-transparent" : undefined}
    >
      <head>
        <HeadContent />
      </head>
      <body
        className={
          isNativeGlassShell
            ? "flex h-full min-h-0 flex-1 flex-col overflow-hidden bg-transparent"
            : undefined
        }
      >
        <QueryClientProvider client={queryClient}>
          {isTrayPanelWindow ? (
            <div className="flex h-full min-h-0 flex-1 flex-col overflow-hidden">
              <TrayEventBridge />
              <Outlet />
            </div>
          ) : (
            <>
              <TrayEventBridge />
              <Outlet />
            </>
          )}
        </QueryClientProvider>
        <Scripts />
      </body>
    </html>
  );
}
