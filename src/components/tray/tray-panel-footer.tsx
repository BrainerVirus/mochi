import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { InfoIcon, LogOutIcon, RefreshCwIcon, SettingsIcon } from "lucide-react";
import { useState, type ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { queryKeys } from "@/lib/query/keys";
import { appVersion } from "@/lib/tauri/commands";
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
  to?: "/settings";
}

function TrayMenuRow({ item, onAbout }: { item: TrayMenuItem; onAbout?: () => void }) {
  const rowClassName = cn(
    "flex w-full cursor-pointer items-center gap-2.5 px-3 py-2 text-left text-sm",
    "hover:bg-secondary/80 active:bg-secondary",
    "disabled:pointer-events-none disabled:opacity-50",
  );

  const content = (
    <>
      <span className="text-muted-foreground flex size-4 shrink-0 items-center justify-center [&_svg]:size-4">
        {item.icon}
      </span>
      <span className="min-w-0 flex-1 truncate">{item.label}</span>
      {item.shortcut ? (
        <span className="text-muted-foreground shrink-0 text-[11px] tabular-nums">
          {item.shortcut}
        </span>
      ) : null}
    </>
  );

  if (item.to) {
    return (
      <li>
        <Link to={item.to} className={rowClassName}>
          {content}
        </Link>
      </li>
    );
  }

  return (
    <li>
      <button
        type="button"
        className={rowClassName}
        disabled={item.disabled}
        onClick={() => {
          if (item.id === "about") {
            onAbout?.();
            return;
          }
          item.onClick?.();
        }}
      >
        {content}
      </button>
    </li>
  );
}

function TrayAboutOverlay({ version, onClose }: { version: string; onClose: () => void }) {
  return (
    <dialog
      open
      className="bg-background/80 absolute inset-0 z-10 m-0 flex max-h-none max-w-none items-center justify-center border-0 p-4 backdrop-blur-[2px]"
      aria-labelledby="tray-about-title"
    >
      <div className="border-border bg-card flex w-full max-w-[280px] flex-col gap-3 rounded-lg border p-4 shadow-sm">
        <div className="flex flex-col gap-1">
          <h2 id="tray-about-title" className="text-sm font-semibold">
            About Mochi
          </h2>
          <p className="text-muted-foreground text-xs">Soft alerts before hard limits.</p>
          <p className="text-foreground text-xs tabular-nums">Version {version}</p>
        </div>
        <Button type="button" size="sm" className="cursor-pointer self-end" onClick={onClose}>
          OK
        </Button>
      </div>
    </dialog>
  );
}

export function TrayPanelFooter({ isRefreshing, onRefresh, onQuit }: TrayPanelFooterProps) {
  const [aboutOpen, setAboutOpen] = useState(false);
  const { data: version = "…" } = useQuery({
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
      to: "/settings",
    },
    {
      id: "about",
      label: "About Mochi",
      icon: <InfoIcon aria-hidden />,
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
    <footer data-tray-panel-footer className="relative shrink-0">
      {aboutOpen ? (
        <TrayAboutOverlay version={version} onClose={() => setAboutOpen(false)} />
      ) : null}
      <nav aria-label="Tray panel actions">
        <ul>
          {items.map((item) => (
            <TrayMenuRow
              key={item.id}
              item={item}
              onAbout={() => {
                setAboutOpen(true);
              }}
            />
          ))}
        </ul>
      </nav>
    </footer>
  );
}
