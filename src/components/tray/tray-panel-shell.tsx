import type { ReactNode, RefObject } from "react";

import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import {
  trayPanelScrollRegionClassName,
  trayPanelShellClassName,
} from "@/lib/utils/tray-panel-layout";

interface TrayPanelShellProps {
  children: ReactNode;
  footer?: ReactNode;
  layoutRef?: RefObject<HTMLDivElement | null>;
}

export function TrayPanelShell({ children, footer, layoutRef }: TrayPanelShellProps) {
  return (
    <main className={trayPanelShellClassName()}>
      <div ref={layoutRef} className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ScrollFadeRegion
          orientation="vertical"
          className={trayPanelScrollRegionClassName()}
          scrollClassName="overscroll-y-contain"
        >
          {children}
        </ScrollFadeRegion>
        {footer ? (
          <>
            <div
              data-tray-panel-separator
              className="border-border shrink-0 border-t"
              aria-hidden
            />
            {footer}
          </>
        ) : null}
      </div>
    </main>
  );
}

export { trayPanelShellClassName };
