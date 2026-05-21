import { ActivityIcon, BarChart3Icon } from "lucide-react";
import type { ReactNode } from "react";

import type { ProviderId } from "@/lib/schemas/usage";

import { getProviderExternalLinks } from "@/lib/providers/dashboard-urls";
import { openExternalUrl } from "@/lib/tauri/commands";
import { getProviderLabel } from "@/lib/utils/provider-labels";
import { cn } from "@/lib/utils";

interface ProviderActionRowProps {
  label: string;
  icon: ReactNode;
  onClick: () => void;
}

function ProviderActionRow({ label, icon, onClick }: ProviderActionRowProps) {
  return (
    <li>
      <button
        type="button"
        className={cn(
          "flex w-full cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-sm",
          "text-muted-foreground hover:bg-secondary/80 hover:text-foreground active:bg-secondary",
        )}
        onClick={onClick}
      >
        <span className="flex size-4 shrink-0 items-center justify-center [&_svg]:size-4">
          {icon}
        </span>
        <span className="min-w-0 flex-1 truncate">{label}</span>
      </button>
    </li>
  );
}

export function ProviderUsageActions({ provider }: { provider: ProviderId }) {
  const links = getProviderExternalLinks(provider);

  if (!links.dashboardUrl && !links.statusPageUrl) {
    return null;
  }

  return (
    <nav aria-label={`${getProviderLabel(provider)} links`}>
      <ul className="flex flex-col gap-0.5">
        {links.dashboardUrl ? (
          <ProviderActionRow
            label="Usage Dashboard"
            icon={<BarChart3Icon aria-hidden />}
            onClick={() => {
              void openExternalUrl(links.dashboardUrl!);
            }}
          />
        ) : null}
        {links.statusPageUrl ? (
          <ProviderActionRow
            label="Status Page"
            icon={<ActivityIcon aria-hidden />}
            onClick={() => {
              void openExternalUrl(links.statusPageUrl!);
            }}
          />
        ) : null}
      </ul>
    </nav>
  );
}
