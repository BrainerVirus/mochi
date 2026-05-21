import { useQuery } from "@tanstack/react-query";
import { InfoIcon, LogOutIcon, RefreshCwIcon, SettingsIcon } from "lucide-react";

import { TrayMenuList, TrayMenuRow, type TrayMenuItem } from "@/components/tray/tray-menu-row";
import { queryKeys } from "@/lib/query/keys";
import { appVersion, openAppWindow } from "@/lib/tauri/commands";
import { trayPanelShortcut } from "@/lib/utils/tray-panel-shortcut";

interface TrayPanelFooterProps {
  isRefreshing: boolean;
  onRefresh: () => void;
  onQuit: () => void;
}

export function TrayPanelFooter({ isRefreshing, onRefresh, onQuit }: TrayPanelFooterProps) {
  useQuery({
    queryKey: queryKeys.appVersion,
    queryFn: appVersion,
    staleTime: Number.POSITIVE_INFINITY,
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
    <div className="pt-0 pb-1" data-tray-panel-footer>
      <TrayMenuList aria-label="Tray panel actions">
        {items.map((item) => (
          <TrayMenuRow key={item.id} item={item} />
        ))}
      </TrayMenuList>
    </div>
  );
}
