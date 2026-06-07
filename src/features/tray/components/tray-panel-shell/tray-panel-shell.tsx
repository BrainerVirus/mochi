import type { ReactNode, RefObject } from "react";

import { ScrollFadeRegion } from "@/features/tray/components/scroll-fade-region";
import {
  trayPanelScrollRegionClassName,
  trayPanelShellClassName,
} from "@/lib/utils/tray-panel-layout";

interface TrayPanelShellProps {
  children: ReactNode;
  layoutRef?: RefObject<HTMLDivElement | null>;
}

export function TrayPanelShell({ children, layoutRef }: TrayPanelShellProps) {
  return (
    <main className={trayPanelShellClassName()}>
      <div ref={layoutRef} className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ScrollFadeRegion
          orientation="vertical"
          controls="none"
          className={trayPanelScrollRegionClassName()}
          scrollClassName="overscroll-y-contain"
        >
          {children}
        </ScrollFadeRegion>
      </div>
    </main>
  );
}
