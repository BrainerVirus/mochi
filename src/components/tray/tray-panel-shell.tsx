import type { ReactNode } from "react";

const trayPanelShellClassName =
  "bg-background text-foreground min-h-full overflow-hidden rounded-mochi shadow-sm ring-1 ring-border";

export function TrayPanelShell({ children }: { children: ReactNode }) {
  return <main className={trayPanelShellClassName}>{children}</main>;
}

export { trayPanelShellClassName };
