import type { ReactNode } from "react";

interface AppWindowShellProps {
  children: ReactNode;
}

/** Native-adjacent shell for dedicated Tauri windows (settings, about). */
export function AppWindowShell({ children }: AppWindowShellProps) {
  return (
    <div className="app-window bg-background text-foreground min-h-svh w-full">{children}</div>
  );
}
