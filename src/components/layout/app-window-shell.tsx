import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface AppWindowShellProps {
  children: ReactNode;
  variant?: "settings" | "about";
}

/** Native-adjacent shell for dedicated Tauri windows (settings, about). */
export function AppWindowShell({ children, variant = "settings" }: AppWindowShellProps) {
  return (
    <div
      className={cn(
        "app-window text-foreground flex h-full min-h-0 w-full flex-1 flex-col overflow-hidden",
        "font-[family-name:var(--font-platform)] antialiased",
        variant === "about" && "app-window--about",
      )}
    >
      {children}
    </div>
  );
}
