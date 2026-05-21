import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

function isMetaShortcut(event: KeyboardEvent): boolean {
  return event.metaKey || event.ctrlKey;
}

interface TrayPanelShortcutHandlers {
  onRefresh: () => void;
  onQuit: () => void;
}

export function useTrayPanelShortcuts({ onRefresh, onQuit }: TrayPanelShortcutHandlers) {
  const navigate = useNavigate();

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
        void navigate({ to: "/settings" });
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
  }, [navigate, onQuit, onRefresh]);
}
