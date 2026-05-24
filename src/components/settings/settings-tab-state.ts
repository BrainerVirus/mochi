import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { shouldHandleAppNavigateEvent } from "@/lib/tauri/window-events";

export const SETTINGS_DEFAULT_TAB = "general";

export function isSettingsRoutePath(path: string): boolean {
  const pathname = path.split("?")[0]?.split("#")[0] ?? "/";
  const normalized = pathname.replace(/\/$/, "") || "/";
  return normalized === "/settings";
}

/** Reset to General whenever the dedicated settings window is opened from the tray. */
export function useSettingsTabState() {
  const [activeTab, setActiveTab] = useState(SETTINGS_DEFAULT_TAB);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    void listen<string>("app-navigate", (event) => {
      if (!shouldHandleAppNavigateEvent()) {
        return;
      }

      if (isSettingsRoutePath(event.payload)) {
        setActiveTab(SETTINGS_DEFAULT_TAB);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  return [activeTab, setActiveTab] as const;
}
