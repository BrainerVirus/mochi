import type { ReactNode } from "react";

import {
  trayPanelScrollRegionClassName,
  trayPanelShellClassName,
} from "@/lib/utils/tray-panel-layout";

export function TrayPanelShell({ children }: { children: ReactNode }) {
  return (
    <main className={trayPanelShellClassName()}>
      <div className={trayPanelScrollRegionClassName()}>{children}</div>
    </main>
  );
}

export { trayPanelShellClassName };
