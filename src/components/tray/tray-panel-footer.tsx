import { useQuery } from "@tanstack/react-query";
import { InfoIcon, LogOutIcon, RefreshCwIcon, SettingsIcon } from "lucide-react";
import { type ReactNode } from "react";

import { queryKeys } from "@/lib/query/keys";
import { appVersion, openAppWindow } from "@/lib/tauri/commands";
import { cn } from "@/lib/utils";
import { trayPanelShortcut } from "@/lib/utils/tray-panel-shortcut";

interface TrayPanelFooterProps {
  isRefreshing: boolean;
  onRefresh: () => void;
  onQuit: () => void;
}

interface TrayMenuItem {
  id: string;
  label: string;
  icon: ReactNode;
  shortcut?: string;
  disabled?: boolean;
  onClick?: () => void;
}

function TrayMenuRow({ item }: { item: TrayMenuItem }) {
  const rowClassName = cn(
    "flex w-full cursor-pointer items-center gap-2.5 px-3 py-2 text-left text-sm",
    "hover:bg-secondary/80 active:bg-secondary",
    "disabled:pointer-events-none disabled:opacity-50",
  );

  return (
    <li>
      <button
        type="button"
        className={rowClassName}
        disabled={item.disabled}
        onClick={item.onClick}
      >
        <span className="text-muted-foreground flex size-4 shrink-0 items-center justify-center [&_svg]:size-4">
          {item.icon}
        </span>
        <span className="min-w-0 flex-1 truncate">{item.label}</span>
        {item.shortcut ? (
          <span className="text-muted-foreground shrink-0 text-[11px] tabular-nums">
            {item.shortcut}
          </span>
        ) : null}
      </button>
    </li>
  );
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
    <div data-tray-panel-footer>
      <nav aria-label="Tray panel actions">
        <ul>
          {items.map((item) => (
            <TrayMenuRow key={item.id} item={item} />
          ))}
        </ul>
      </nav>
    </div>
  );
}
