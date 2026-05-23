import { QueryClientProvider } from "@tanstack/react-query";
import { HeadContent, Outlet, Scripts, createRootRoute } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";

import { TrayEventBridge } from "@/components/tray/tray-event-bridge";
import { detectPlatform, useSystemColorScheme, type PlatformId } from "@/lib/platform";
import { queryClient } from "@/lib/query/client";

import appCss from "@/styles/index.css?url";

const TRAY_PANEL_WINDOW_LABEL = "main";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: "Mochi" },
    ],
    links: [{ rel: "stylesheet", href: appCss }],
  }),
  component: RootComponent,
});

function RootComponent() {
  const [isTrayPanelWindow, setIsTrayPanelWindow] = useState(false);
  const [platform, setPlatform] = useState<PlatformId>("unknown");

  useSystemColorScheme();

  useEffect(() => {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
      return;
    }

    try {
      const webviewWindow = getCurrentWebviewWindow();
      setIsTrayPanelWindow(webviewWindow.label === TRAY_PANEL_WINDOW_LABEL);
    } catch {
      setIsTrayPanelWindow(false);
    }
  }, []);

  useEffect(() => {
    void detectPlatform().then(setPlatform);
  }, []);

  return (
    <html
      lang="en"
      data-platform={platform}
      className={isTrayPanelWindow ? "h-full bg-transparent" : undefined}
    >
      <head>
        <HeadContent />
      </head>
      <body
        className={
          isTrayPanelWindow
            ? "flex h-full min-h-0 flex-col overflow-hidden bg-transparent"
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
