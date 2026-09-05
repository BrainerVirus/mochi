import { useNavigate } from "@tanstack/react-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";

import { takePendingAppRoute } from "@/lib/tauri/commands";
import {
  initialRouteForWindowLabel,
  shouldNavigateFromPackagedShell,
} from "@/lib/tauri/initial-window-route";

/** Navigates packaged Tauri webviews from the static shell to their window-specific route. */
export function useInitialWindowRoute() {
  const navigate = useNavigate();

  useEffect(() => {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
      return;
    }

    const bootFromPackagedShell = async () => {
      try {
        const label = getCurrentWebviewWindow().label;
        const pathname = window.location.pathname;
        if (!shouldNavigateFromPackagedShell(pathname)) {
          return;
        }

        // Fresh windows boot after the open_app_window event fires, so the
        // live listener misses it — the stored route survives the boot.
        const pending = await takePendingAppRoute();
        if (typeof pending === "string" && pending.length > 0) {
          if (pathname === pending) {
            return;
          }

          await navigate({ to: pending, replace: true });
          return;
        }

        const target = initialRouteForWindowLabel(label);
        if (pathname === target) {
          return;
        }

        await navigate({ to: target, replace: true });
      } catch {
        // Ignore when the webview API is unavailable (e.g. unit tests).
      }
    };

    void bootFromPackagedShell();
  }, [navigate]);
}
