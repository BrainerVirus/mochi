import { RefreshCwIcon } from "lucide-react";

import { rateWindows, type ProviderId, type UsageSnapshot } from "@/lib/schemas/usage";

import { ProviderIcon } from "@/components/providers/provider-icon";
import { TrayPanelDivider } from "@/components/tray/tray-panel-divider";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatUpdatedAgo } from "@/lib/utils/format-updated-ago";
import { getProviderLabel } from "@/lib/utils/provider-labels";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";

import { reserveDetailLeft, reserveDetailRight } from "@/lib/utils/usage-pace";

import { ProviderCostSection } from "./provider-cost-section";
import { ProviderUsageActions } from "./provider-usage-actions";
import { UsageMeter } from "./usage-meter";

function UsageWindowMeters({
  windows,
  fillActivationKey,
}: {
  windows: ReturnType<typeof rateWindows>;
  fillActivationKey?: string;
}) {
  return (
    <>
      {windows.map((window) => (
        <UsageMeter
          key={`${fillActivationKey ?? "static"}-${window.label}`}
          label={window.label}
          usedPercent={window.used_percent}
          remainingPercent={window.remaining_percent}
          resetsAt={window.resets_at}
          detailLeft={reserveDetailLeft(window)}
          detailRight={reserveDetailRight(window)}
          compact
          fillActivationKey={fillActivationKey}
        />
      ))}
    </>
  );
}

interface ProviderUsageSectionProps {
  snapshot: UsageSnapshot;
  onRefresh?: (provider: ProviderId) => void;
  isRefreshing?: boolean;
  planLabel?: string | null;
  showProviderActions?: boolean;
  fillActivationKey?: string;
}

export function ProviderUsageSection({
  snapshot,
  onRefresh,
  isRefreshing = false,
  planLabel = null,
  showProviderActions = false,
  fillActivationKey,
}: ProviderUsageSectionProps) {
  const windows = rateWindows(snapshot);

  return (
    <section className="flex flex-col">
      <header className="flex items-start justify-between gap-2">
        <div className="flex min-w-0 flex-col gap-0.5">
          <h3 className="flex items-center gap-1.5 text-sm font-semibold">
            <ProviderIcon provider={snapshot.provider} className="size-4" />
            <span className="truncate">{getProviderLabel(snapshot.provider)}</span>
            {planLabel ? (
              <Badge variant="outline" className="text-[10px] font-normal">
                {planLabel}
              </Badge>
            ) : null}
          </h3>
          <p className="text-muted-foreground text-[11px]">
            {formatUpdatedAgo(snapshot.updated_at)}
          </p>
        </div>
        {onRefresh ? (
          <Button
            variant="ghost"
            size="icon-sm"
            className="shrink-0 cursor-pointer"
            disabled={isRefreshing}
            aria-label={`Refresh ${getProviderLabel(snapshot.provider)} usage`}
            onClick={() => {
              onRefresh(snapshot.provider);
            }}
          >
            <RefreshCwIcon
              data-icon="inline-start"
              className={isRefreshing ? "animate-spin" : undefined}
            />
          </Button>
        ) : null}
      </header>
      <div
        className={`${trayPanelSpacing.headerToMeters} flex flex-col ${trayPanelSpacing.meterGap}`}
      >
        {snapshot.error ? (
          <p className="text-destructive text-[11px] leading-snug">{snapshot.error}</p>
        ) : null}
        <UsageWindowMeters windows={windows} fillActivationKey={fillActivationKey} />
        {snapshot.provider_cost ? (
          <ProviderCostSection
            cost={snapshot.provider_cost}
            compact
            fillActivationKey={fillActivationKey}
          />
        ) : null}
      </div>
      {showProviderActions ? (
        <>
          <TrayPanelDivider />
          <ProviderUsageActions provider={snapshot.provider} />
        </>
      ) : null}
    </section>
  );
}
