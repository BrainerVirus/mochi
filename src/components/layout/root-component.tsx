import { QueryClientProvider } from "@tanstack/react-query";
import { HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { TrayEventBridge } from "@/components/tray/tray-event-bridge";
import {
  detectPlatform,
  detectPlatformFromNavigator,
  useSystemColorScheme,
  type PlatformId,
} from "@/lib/platform";
import { queryClient } from "@/lib/query/client";
import { readIsTrayPanelWindow } from "@/lib/tauri/tray-panel-window";

export function RootComponent() {
  const [isTrayPanelWindow, setIsTrayPanelWindow] = useState(readIsTrayPanelWindow);
  const [platform, setPlatform] = useState<PlatformId>(detectPlatformFromNavigator);

  useSystemColorScheme(!isTrayPanelWindow);

  useEffect(() => {
    setIsTrayPanelWindow(readIsTrayPanelWindow());
    void detectPlatform().then(setPlatform);
  }, []);

  return (
    <html
      lang="en"
      data-platform={platform}
      data-tray-panel={isTrayPanelWindow ? "" : undefined}
      className={isTrayPanelWindow ? "h-full bg-transparent" : undefined}
    >
      <head>
        <HeadContent />
      </head>
      <body
        className={
          isTrayPanelWindow
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
