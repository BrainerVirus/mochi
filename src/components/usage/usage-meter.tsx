import { useRef } from "react";

import { Progress } from "@/components/ui/progress";
import { useUsageMeterFill } from "@/hooks/use-usage-meter-fill";
import { useUsageMeterLeftLabel } from "@/hooks/use-usage-meter-left-label";
import { cn } from "@/lib/utils/cn";
import { formatResetLine } from "@/lib/utils/format-reset-line";
import {
  formatUsageMeterLeftLabel,
  resolveUsageMeterDisplayPercent,
} from "@/lib/utils/usage-meter-fill-animation";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

interface UsageMeterProps {
  label: string;
  usedPercent: number;
  remainingPercent?: number;
  resetsAt?: string | null;
  compact?: boolean;
  detailLeft?: string | null;
  detailRight?: string | null;
  /** Changes when tray tab content activates; retriggers fill from empty. */
  fillActivationKey?: string;
}

export function UsageMeter({
  label,
  usedPercent,
  remainingPercent,
  resetsAt = null,
  compact = false,
  detailLeft = null,
  detailRight = null,
  fillActivationKey = "static",
}: UsageMeterProps) {
  const clamped = Math.max(0, Math.min(100, usedPercent));
  const leftPercent = resolveUsageMeterDisplayPercent(clamped, remainingPercent);
  const tone = getUsageMeterTone(clamped);
  const resetLabel = formatResetLine(resetsAt);
  const meterRef = useRef<HTMLDivElement>(null);
  const indicatorRef = useRef<HTMLDivElement>(null);
  const leftLabelRef = useRef<HTMLSpanElement>(null);

  useUsageMeterFill(meterRef, indicatorRef, leftPercent, fillActivationKey);
  useUsageMeterLeftLabel(leftLabelRef, leftPercent, fillActivationKey);

  return (
    <div ref={meterRef} className={cn("flex flex-col", compact ? "gap-1" : "gap-1.5")}>
      <span className={cn("font-medium", compact ? "text-xs" : "text-sm")}>{label}</span>
      <Progress
        value={leftPercent}
        indicatorRef={indicatorRef}
        animateIndicator
        className={cn(compact ? "h-1" : "h-1.5", usageMeterToneClasses[tone])}
        aria-label={`${label}: ${Math.round(leftPercent)}% remaining`}
      />
      <div
        className={cn(
          "flex items-baseline justify-between gap-2",
          compact ? "text-[11px]" : "text-xs",
        )}
      >
        <span ref={leftLabelRef} className="text-foreground font-medium tabular-nums">
          {formatUsageMeterLeftLabel(0)}
        </span>
        {resetLabel ? (
          <span className="text-muted-foreground tabular-nums">{resetLabel}</span>
        ) : null}
      </div>
      {detailLeft || detailRight ? (
        <div
          className={cn(
            "flex items-baseline justify-between gap-2",
            compact ? "text-[10px]" : "text-[11px]",
          )}
        >
          {detailLeft ? (
            <span className="text-muted-foreground truncate">{detailLeft}</span>
          ) : (
            <span />
          )}
          {detailRight ? (
            <span className="text-muted-foreground shrink-0 tabular-nums">{detailRight}</span>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
