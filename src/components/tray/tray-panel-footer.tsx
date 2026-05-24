import { useQuery } from "@tanstack/react-query";
import { InfoIcon, LogOutIcon, RefreshCwIcon, SettingsIcon } from "lucide-react";

import { TrayMenuList, TrayMenuRow, type TrayMenuItem } from "@/components/tray/tray-menu-row";
import { useUpdateCheck } from "@/hooks/use-update-install";
import { queryKeys } from "@/lib/query/keys";
import { appVersion, openAppWindow } from "@/lib/tauri/commands";
import { buildTrayUpdateFooterItems } from "@/lib/updates/tray-update-footer-items";
import { trayPanelShortcut } from "@/lib/utils/tray-panel-shortcut";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";

interface TrayPanelFooterProps {
  isRefreshing: boolean;
  onRefresh: () => void;
  onQuit: () => void;
}

export function TrayPanelFooter({ isRefreshing, onRefresh, onQuit }: TrayPanelFooterProps) {
  const { data: updateInfo } = useUpdateCheck();
  useQuery({
    queryKey: queryKeys.appVersion,
    queryFn: appVersion,
    staleTime: Number.POSITIVE_INFINITY,
  });

  const updateItems = buildTrayUpdateFooterItems({
    updateAvailable: updateInfo?.available ?? false,
    updateVersion: updateInfo?.version,
    onOpenUpdate: () => {
      void openAppWindow("/update");
    },
    onCheckUpdates: () => {
      void openAppWindow("/update");
    },
  });

  const items: TrayMenuItem[] = [
    {
      id: "refresh",
      label: "Refresh",
      icon: <RefreshCwIcon className={isRefreshing ? "animate-spin" : undefined} aria-hidden />,
      shortcut: trayPanelShortcut("R"),
      disabled: isRefreshing,
      onClick: onRefresh,
    },
    {
      id: "settings",
      label: "Settings",
      icon: <SettingsIcon aria-hidden />,
      shortcut: trayPanelShortcut(","),
      onClick: () => {
        void openAppWindow("/settings");
      },
    },
    ...updateItems,
    {
      id: "about",
      label: "About Mochi",
      icon: <InfoIcon aria-hidden />,
      onClick: () => {
        void openAppWindow("/about");
      },
    },
    {
      id: "quit",
      label: "Quit",
      icon: <LogOutIcon aria-hidden />,
      shortcut: trayPanelShortcut("Q"),
      onClick: onQuit,
    },
  ];

  return (
    <div className={`pt-0 ${trayPanelSpacing.footerBottom}`} data-tray-panel-footer>
      <TrayMenuList aria-label="Tray panel actions">
        {items.map((item) => (
          <TrayMenuRow key={item.id} item={item} />
        ))}
      </TrayMenuList>
    </div>
  );
}
