import { ActivityIcon, BarChart3Icon } from "lucide-react";

import { TrayMenuList, TrayMenuRow, type TrayMenuItem } from "@/components/tray/tray-menu-row";
import type { ProviderId } from "@/lib/schemas/usage";
import { getProviderExternalLinks } from "@/lib/providers/dashboard-urls";
import { openExternalUrl } from "@/lib/tauri/commands";
import { getProviderLabel } from "@/lib/utils/provider-labels";

export function ProviderUsageActions({ provider }: { provider: ProviderId }) {
  const links = getProviderExternalLinks(provider);

  const items: TrayMenuItem[] = [];

  if (links.dashboardUrl) {
    items.push({
      id: "dashboard",
      label: "Usage Dashboard",
      icon: <BarChart3Icon aria-hidden />,
      onClick: () => {
        void openExternalUrl(links.dashboardUrl!);
      },
    });
  }

  if (links.statusPageUrl) {
    items.push({
      id: "status",
      label: "Status Page",
      icon: <ActivityIcon aria-hidden />,
      onClick: () => {
        void openExternalUrl(links.statusPageUrl!);
      },
    });
  }

  if (items.length === 0) {
    return null;
  }

  return (
    <div className="pt-2">
      <div className="bg-border mb-1 h-px w-full" aria-hidden />
      <div className="-mx-3">
        <TrayMenuList aria-label={`${getProviderLabel(provider)} links`}>
          {items.map((item) => (
            <TrayMenuRow key={item.id} item={item} />
          ))}
        </TrayMenuList>
      </div>
    </div>
  );
}
