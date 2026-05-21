import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/utils";

export interface TrayMenuItem {
  id: string;
  label: string;
  icon: ReactNode;
  shortcut?: string;
  disabled?: boolean;
  onClick?: () => void;
}

export function TrayMenuRow({
  item,
  className,
  ...props
}: { item: TrayMenuItem } & ComponentProps<"li">) {
  return (
    <li className={className} {...props}>
      <button
        type="button"
        className={cn(
          "flex h-7 w-full cursor-pointer items-center gap-2 rounded-md px-3 py-1.5 text-left text-sm",
          "text-foreground hover:bg-white/10 active:bg-white/15",
          "disabled:pointer-events-none disabled:opacity-50",
        )}
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

export function TrayMenuList({
  "aria-label": ariaLabel,
  children,
}: {
  "aria-label": string;
  children: ReactNode;
}) {
  return (
    <nav aria-label={ariaLabel}>
      <ul>{children}</ul>
    </nav>
  );
}
