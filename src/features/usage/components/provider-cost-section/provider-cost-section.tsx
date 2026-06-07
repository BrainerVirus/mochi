import type { ProviderCostSnapshot } from "@/lib/schemas/usage";

import { useRef } from "react";
import { Progress } from "@/components/ui/progress";
import { useUsageMeterFill } from "@/features/usage/hooks/use-usage-meter-fill/use-usage-meter-fill";
import { useUsageMeterLeftLabel } from "@/features/usage/hooks/use-usage-meter-left-label/use-usage-meter-left-label";
import { cn } from "@/lib/utils";
import { formatResetLine } from "@/lib/utils/format-reset-line";
import { formatUsageMeterLeftLabel } from "@/lib/utils/usage-meter-fill-animation";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

interface ProviderCostSectionProps {
  cost: ProviderCostSnapshot;
  compact?: boolean;
  fillActivationKey?: string;
}

function formatCurrency(amount: number, currencyCode: string): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency: currencyCode,
      maximumFractionDigits: 2,
    }).format(amount);
  } catch {
    return `$${amount.toFixed(2)}`;
  }
}

export function ProviderCostSection({
  cost,
  compact = false,
  fillActivationKey = "static",
}: ProviderCostSectionProps) {
  const label = cost.period ?? "On-demand";
  const hasLimit = cost.limit > 0;
  const usedPercent = hasLimit ? Math.max(0, Math.min(100, (cost.used / cost.limit) * 100)) : 0;
  const leftPercent = hasLimit ? Math.max(0, Math.min(100, 100 - usedPercent)) : 100;
  const tone = getUsageMeterTone(usedPercent);
  const resetLabel = formatResetLine(cost.resets_at ?? null);
  const meterRef = useRef<HTMLDivElement>(null);
  const indicatorRef = useRef<HTMLDivElement>(null);
  const leftLabelRef = useRef<HTMLSpanElement>(null);

  useUsageMeterFill(meterRef, indicatorRef, leftPercent, fillActivationKey);
  useUsageMeterLeftLabel(leftLabelRef, leftPercent, fillActivationKey);

  const detail = hasLimit
    ? `${formatCurrency(cost.used, cost.currency_code)} / ${formatCurrency(cost.limit, cost.currency_code)}`
    : formatCurrency(cost.used, cost.currency_code);

  return (
    <div ref={meterRef} className={cn("flex flex-col", compact ? "gap-1" : "gap-1.5")}>
      <span className={cn("font-medium", compact ? "text-xs" : "text-sm")}>{label}</span>
      {hasLimit ? (
        <Progress
          value={leftPercent}
          indicatorRef={indicatorRef}
          animateIndicator
          className={cn(compact ? "h-1" : "h-1.5", usageMeterToneClasses[tone])}
          aria-label={`${label}: ${Math.round(leftPercent)}% remaining`}
        />
      ) : null}
      <div
        className={cn(
          "flex items-baseline justify-between gap-2",
          compact ? "text-[11px]" : "text-xs",
        )}
      >
        <span ref={leftLabelRef} className="text-foreground font-medium tabular-nums">
          {hasLimit ? formatUsageMeterLeftLabel(0) : detail}
        </span>
        {resetLabel ? (
          <span className="text-muted-foreground tabular-nums">{resetLabel}</span>
        ) : null}
      </div>
      {hasLimit ? (
        <p
          className={cn(
            "text-muted-foreground tabular-nums",
            compact ? "text-[10px]" : "text-[11px]",
          )}
        >
          {detail}
        </p>
      ) : null}
    </div>
  );
}
