import type { ReactNode } from "react";

import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import {
  trayPanelScrollRegionClassName,
  trayPanelShellClassName,
} from "@/lib/utils/tray-panel-layout";

export function TrayPanelShell({ children }: { children: ReactNode }) {
  return (
    <main className={trayPanelShellClassName()}>
      <ScrollFadeRegion
        orientation="vertical"
        className={trayPanelScrollRegionClassName()}
        scrollClassName="overscroll-y-contain"
      >
        {children}
      </ScrollFadeRegion>
    </main>
  );
}

export { trayPanelShellClassName };
