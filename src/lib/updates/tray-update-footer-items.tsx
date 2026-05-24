import { ArrowDownCircleIcon, DownloadIcon } from "lucide-react";

import type { TrayMenuItem } from "@/components/tray/tray-menu-row";

export interface TrayUpdateFooterOptions {
  updateAvailable: boolean;
  updateVersion?: string | null;
  onOpenUpdate: () => void;
  onCheckUpdates: () => void;
}

export function buildTrayUpdateFooterItems({
  updateAvailable,
  updateVersion,
  onOpenUpdate,
  onCheckUpdates,
}: TrayUpdateFooterOptions): TrayMenuItem[] {
  if (updateAvailable) {
    const versionLabel = updateVersion ? ` v${updateVersion}` : "";
    return [
      {
        id: "update-available",
        label: `Update available${versionLabel}`,
        icon: <ArrowDownCircleIcon aria-hidden />,
        highlight: true,
        onClick: onOpenUpdate,
      },
    ];
  }

  return [
    {
      id: "check-updates",
      label: "Check for updates",
      icon: <DownloadIcon aria-hidden />,
      onClick: onCheckUpdates,
    },
  ];
}
