import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

interface UsageMeterProps {
  label: string;
  usedPercent: number;
}

export function UsageMeter({ label, usedPercent }: UsageMeterProps) {
  const clamped = Math.max(0, Math.min(100, usedPercent));
  const tone = getUsageMeterTone(clamped);

  return (
    <div className="flex flex-col gap-1">
      <div className="text-muted-foreground flex items-center justify-between text-xs">
        <span>{label}</span>
        <span className="font-medium tabular-nums">{Math.round(clamped)}% used</span>
      </div>
      <Progress
        value={clamped}
        className={cn("h-2", usageMeterToneClasses[tone])}
        aria-label={`${label}: ${Math.round(clamped)}% used`}
      />
    </div>
  );
}
