import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface AppWindowShellProps {
  children: ReactNode;
  variant?: "settings" | "about" | "update";
}

/** Native-adjacent shell for dedicated Tauri windows (settings, about, update). */
export function AppWindowShell({ children, variant = "settings" }: AppWindowShellProps) {
  return (
    <div
      className={cn(
        "app-window text-foreground flex h-full min-h-0 w-full flex-1 flex-col overflow-hidden",
        "font-[family-name:var(--font-platform)] antialiased",
        variant === "about" && "app-window--about",
        variant === "update" && "app-window--update",
      )}
    >
      <div className="app-window-titlebar">
        <div className="app-window-titlebar__drag" data-tauri-drag-region aria-hidden="true" />
        <span className="app-window-titlebar__title">Mochi</span>
      </div>
      <div className="app-window-body flex min-h-0 flex-1 flex-col overflow-hidden">{children}</div>
    </div>
  );
}
