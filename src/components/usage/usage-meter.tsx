interface UsageMeterProps {
  label: string;
  usedPercent: number;
}

export function UsageMeter({ label, usedPercent }: UsageMeterProps) {
  const clamped = Math.max(0, Math.min(100, usedPercent));

  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center justify-between text-xs text-slate-600">
        <span>{label}</span>
        <span>{Math.round(clamped)}% used</span>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-slate-200">
        <div
          className="bg-mochi-blush h-full rounded-full transition-all duration-300"
          style={{ width: `${clamped}%` }}
        />
      </div>
    </div>
  );
}
