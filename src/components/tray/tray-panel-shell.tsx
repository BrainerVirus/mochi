import type { ReactNode } from "react";

import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import {
  trayPanelScrollRegionClassName,
  trayPanelShellClassName,
} from "@/lib/utils/tray-panel-layout";

interface TrayPanelShellProps {
  children: ReactNode;
  footer?: ReactNode;
}

export function TrayPanelShell({ children, footer }: TrayPanelShellProps) {
  return (
    <main className={trayPanelShellClassName()}>
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ScrollFadeRegion
          orientation="vertical"
          className={trayPanelScrollRegionClassName()}
          scrollClassName="overscroll-y-contain"
        >
          {children}
        </ScrollFadeRegion>
        {footer}
      </div>
    </main>
  );
}

export { trayPanelShellClassName };
