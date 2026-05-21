import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { formatResetCountdown } from "@/lib/utils/format-reset-countdown";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

interface UsageMeterProps {
  label: string;
  usedPercent: number;
  remainingPercent?: number;
  resetsAt?: string | null;
  compact?: boolean;
}

export function UsageMeter({
  label,
  usedPercent,
  remainingPercent,
  resetsAt = null,
  compact = false,
}: UsageMeterProps) {
  const clamped = Math.max(0, Math.min(100, usedPercent));
  const leftPercent =
    remainingPercent !== undefined
      ? Math.max(0, Math.min(100, remainingPercent))
      : Math.max(0, 100 - clamped);
  const tone = getUsageMeterTone(clamped);
  const resetLabel = formatResetCountdown(resetsAt);

  return (
    <div className={cn("flex flex-col", compact ? "gap-0.5" : "gap-1")}>
      <div
        className={cn(
          "flex items-center justify-between gap-2",
          compact ? "text-[11px]" : "text-xs",
          "text-muted-foreground",
        )}
      >
        <span className="truncate">{label}</span>
        <span className="flex shrink-0 items-center gap-1.5 tabular-nums">
          <span className="text-foreground font-medium">{Math.round(leftPercent)}% left</span>
          {resetLabel ? (
            <>
              <span aria-hidden="true" className="text-border">
                ·
              </span>
              <span>{resetLabel}</span>
            </>
          ) : null}
        </span>
      </div>
      <Progress
        value={clamped}
        className={cn(compact ? "h-1" : "h-1.5", usageMeterToneClasses[tone])}
        aria-label={`${label}: ${Math.round(leftPercent)}% remaining`}
      />
    </div>
  );
}
