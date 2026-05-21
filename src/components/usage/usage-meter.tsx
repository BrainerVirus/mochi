import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { useRef } from "react";

import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { animateUsageMeterFill } from "@/lib/utils/usage-meter-fill-animation";
import { formatResetLine } from "@/lib/utils/format-reset-line";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

gsap.registerPlugin(useGSAP);

interface UsageMeterProps {
  label: string;
  usedPercent: number;
  remainingPercent?: number;
  resetsAt?: string | null;
  compact?: boolean;
  detailLeft?: string | null;
  detailRight?: string | null;
}

export function UsageMeter({
  label,
  usedPercent,
  remainingPercent,
  resetsAt = null,
  compact = false,
  detailLeft = null,
  detailRight = null,
}: UsageMeterProps) {
  const clamped = Math.max(0, Math.min(100, usedPercent));
  const leftPercent =
    remainingPercent !== undefined
      ? Math.max(0, Math.min(100, remainingPercent))
      : Math.max(0, 100 - clamped);
  const tone = getUsageMeterTone(clamped);
  const resetLabel = formatResetLine(resetsAt);
  const meterRef = useRef<HTMLDivElement>(null);
  const indicatorRef = useRef<HTMLDivElement>(null);
  const previousPercentRef = useRef<number | null>(null);

  useGSAP(
    () => {
      const indicator = indicatorRef.current;
      if (!indicator) {
        return;
      }

      const from = previousPercentRef.current ?? 0;
      animateUsageMeterFill(indicator, from, clamped);
      previousPercentRef.current = clamped;
    },
    { dependencies: [clamped], scope: meterRef },
  );

  return (
    <div ref={meterRef} className={cn("flex flex-col", compact ? "gap-1" : "gap-1.5")}>
      <span className={cn("font-medium", compact ? "text-xs" : "text-sm")}>{label}</span>
      <Progress
        value={clamped}
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
        <span className="text-foreground font-medium tabular-nums">
          {Math.round(leftPercent)}% left
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
