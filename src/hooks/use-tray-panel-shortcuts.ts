import { useEffect } from "react";

import { openAppWindow } from "@/lib/tauri/commands";

function isMetaShortcut(event: KeyboardEvent): boolean {
  return event.metaKey || event.ctrlKey;
}

interface TrayPanelShortcutHandlers {
  onRefresh: () => void;
  onQuit: () => void;
}

export function useTrayPanelShortcuts({ onRefresh, onQuit }: TrayPanelShortcutHandlers) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!isMetaShortcut(event) || event.altKey) {
        return;
      }

      const key = event.key.toLowerCase();

      if (key === "r") {
        event.preventDefault();
        onRefresh();
        return;
      }

      if (key === ",") {
        event.preventDefault();
        void openAppWindow("/settings");
        return;
      }

      if (key === "q") {
        event.preventDefault();
        onQuit();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [onQuit, onRefresh]);
}
