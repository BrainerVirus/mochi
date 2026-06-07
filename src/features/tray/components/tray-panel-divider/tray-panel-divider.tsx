import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";
import { trayPanelDividerClassName, trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";

type TrayPanelDividerProps = ComponentProps<"div"> & {
  /** Adds horizontal inset when the divider sits outside a padded content column. */
  inset?: boolean;
};

export function TrayPanelDivider({ inset = false, className, ...props }: TrayPanelDividerProps) {
  return (
    <div className={cn(trayPanelDividerClassName(inset), className)} {...props} aria-hidden>
      <div className={`bg-border h-px w-full ${trayPanelSpacing.dividerAfter}`} />
    </div>
  );
}
