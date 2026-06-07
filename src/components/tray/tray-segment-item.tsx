import { LayoutGridIcon } from "lucide-react";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { ProviderIcon } from "@/components/providers/provider-icon";
import { ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

const traySegmentItemClassName = cn(
  "relative z-10 inline-flex h-full min-w-[4.75rem] max-w-[7.5rem] shrink-0 flex-none cursor-pointer flex-row items-center justify-center gap-1.5 rounded-none border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:bg-transparent hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground data-[state=on]:shadow-none",
  "first:rounded-none last:rounded-none",
);

export function TraySegmentItem({
  tab,
  setItemRef,
  onHover,
}: {
  tab: TrayPanelTab;
  setItemRef: (id: string, element: HTMLButtonElement | null) => void;
  onHover: (id: string) => void;
}) {
  return (
    <ToggleGroupItem
      ref={(element) => {
        setItemRef(tab.id, element);
      }}
      data-tray-tab-id={tab.id}
      value={tab.id}
      aria-label={tab.label}
      className={traySegmentItemClassName}
      onPointerEnter={() => onHover(tab.id)}
    >
      {tab.id === "overview" ? (
        <LayoutGridIcon className="size-4 shrink-0 opacity-90" aria-hidden />
      ) : (
        <ProviderIcon provider={tab.id} />
      )}
      <span className="min-w-0 truncate text-xs font-medium">{tab.label}</span>
    </ToggleGroupItem>
  );
}
