import { QueryClientProvider } from "@tanstack/react-query";
import { HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";

import { TrayEventBridge } from "@/components/tray/tray-event-bridge";
import {
  detectPlatform,
  detectPlatformFromNavigator,
  useSystemColorScheme,
  type PlatformId,
} from "@/lib/platform";
import { queryClient } from "@/lib/query/client";

const TRAY_PANEL_WINDOW_LABEL = "main";

function readIsTrayPanelWindow(): boolean {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return false;
  }

  try {
    return getCurrentWebviewWindow().label === TRAY_PANEL_WINDOW_LABEL;
  } catch {
    return false;
  }
}

export function RootComponent() {
  const [isTrayPanelWindow] = useState(readIsTrayPanelWindow);
  const [platform, setPlatform] = useState<PlatformId>(detectPlatformFromNavigator);

  useSystemColorScheme();

  useEffect(() => {
    void detectPlatform().then(setPlatform);
  }, []);

  return (
    <html
      lang="en"
      data-platform={platform}
      className={isTrayPanelWindow ? "h-auto bg-transparent" : undefined}
    >
      <head>
        <HeadContent />
      </head>
      <body
        className={
          isTrayPanelWindow
            ? "flex h-auto min-h-0 flex-col overflow-hidden bg-transparent"
            : undefined
        }
      >
        <QueryClientProvider client={queryClient}>
          <TrayEventBridge />
          <Outlet />
        </QueryClientProvider>
        <Scripts />
      </body>
    </html>
  );
}
