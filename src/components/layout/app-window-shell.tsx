import type { ReactNode } from "react";

interface AppWindowShellProps {
  children: ReactNode;
}

/** Cream Mochi shell for dedicated Tauri windows (settings, about). */
export function AppWindowShell({ children }: AppWindowShellProps) {
  return <div className="bg-background text-foreground min-h-svh w-full">{children}</div>;
}
