import { RefreshCwIcon } from "lucide-react";

import {
  rateWindows,
  type ProviderId,
  type ProviderUsageState,
  type UsageSnapshot,
} from "@/lib/schemas/usage";

import { ProviderIcon } from "@/components/providers/provider-icon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { TrayPanelDivider } from "@/features/tray/components/tray-panel-divider";
import { formatUpdatedAgo } from "@/lib/utils/format-updated-ago";
import { getProviderLabel } from "@/lib/utils/provider-labels";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";

import { reserveDetailLeft, reserveDetailRight } from "@/lib/utils/usage-pace";

import { ProviderCostSection } from "@/features/usage/components/provider-cost-section";
import { ProviderUsageActions } from "@/features/usage/components/provider-usage-actions";
import { UsageMeter } from "@/features/usage/components/usage-meter";

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
  snapshot?: UsageSnapshot;
  state?: ProviderUsageState;
  onRefresh?: (provider: ProviderId) => void;
  isRefreshing?: boolean;
  planLabel?: string | null;
  showProviderActions?: boolean;
  fillActivationKey?: string;
}

export function ProviderUsageSection({
  snapshot,
  state,
  onRefresh,
  isRefreshing = false,
  planLabel = null,
  showProviderActions = false,
  fillActivationKey,
}: ProviderUsageSectionProps) {
  const provider = snapshot?.provider ?? state?.provider;
  if (!provider) {
    return null;
  }

  const windows = snapshot ? rateWindows(snapshot) : [];
  const updatedAt = snapshot?.updated_at ?? state?.updated_at;
  const message = state?.message ?? snapshot?.error;
  const isFetching = state?.kind === "fetching";

  return (
    <section className="flex flex-col">
      <header className="flex items-start justify-between gap-2">
        <div className="flex min-w-0 flex-col gap-0.5">
          <h3 className="flex items-center gap-1.5 text-sm font-semibold">
            <ProviderIcon provider={provider} className="size-4" />
            <span className="truncate">{getProviderLabel(provider)}</span>
            {planLabel ? (
              <Badge variant="outline" className="text-[10px] font-normal">
                {planLabel}
              </Badge>
            ) : null}
          </h3>
          {updatedAt ? (
            <p className="text-muted-foreground text-[11px]">{formatUpdatedAgo(updatedAt)}</p>
          ) : null}
        </div>
        {onRefresh ? (
          <Button
            variant="ghost"
            size="icon-sm"
            className="shrink-0 cursor-pointer"
            disabled={isRefreshing}
            aria-label={`Refresh ${getProviderLabel(provider)} usage`}
            onClick={() => {
              onRefresh(provider);
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
        <ProviderUsageBody
          message={message}
          isFetching={isFetching}
          snapshot={snapshot}
          windows={windows}
          fillActivationKey={fillActivationKey}
        />
      </div>
      {showProviderActions ? (
        <>
          <TrayPanelDivider />
          <ProviderUsageActions provider={provider} />
        </>
      ) : null}
    </section>
  );
}

function ProviderUsageBody({
  message,
  isFetching,
  snapshot,
  windows,
  fillActivationKey,
}: {
  message: string | null | undefined;
  isFetching: boolean;
  snapshot: UsageSnapshot | undefined;
  windows: ReturnType<typeof rateWindows>;
  fillActivationKey: string | undefined;
}) {
  return (
    <>
      {message ? <p className="text-muted-foreground text-[11px] leading-snug">{message}</p> : null}
      {isFetching ? <ProviderUsageSkeleton /> : null}
      {snapshot ? (
        <UsageWindowMeters windows={windows} fillActivationKey={fillActivationKey} />
      ) : null}
      {snapshot?.provider_cost ? (
        <ProviderCostSection
          cost={snapshot.provider_cost}
          compact
          fillActivationKey={fillActivationKey}
        />
      ) : null}
    </>
  );
}

function ProviderUsageSkeleton() {
  return (
    <div className="flex flex-col gap-1.5" aria-hidden="true">
      <Skeleton className="h-1.5 w-full rounded-full" />
      <div className="flex items-center justify-between gap-2">
        <Skeleton className="h-3 w-16" />
        <Skeleton className="h-3 w-10" />
      </div>
    </div>
  );
}
