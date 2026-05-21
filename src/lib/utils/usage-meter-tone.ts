export type UsageMeterTone = "normal" | "warning" | "critical";

const WARNING_THRESHOLD = 60;
const CRITICAL_THRESHOLD = 85;

export function getUsageMeterTone(usedPercent: number): UsageMeterTone {
  const clamped = Math.max(0, Math.min(100, usedPercent));

  if (clamped >= CRITICAL_THRESHOLD) {
    return "critical";
  }

  if (clamped >= WARNING_THRESHOLD) {
    return "warning";
  }

  return "normal";
}

export const usageMeterToneClasses: Record<UsageMeterTone, string> = {
  normal: "[&_[data-slot=progress-indicator]]:bg-mochi-matcha",
  warning: "[&_[data-slot=progress-indicator]]:bg-mochi-yuzu",
  critical: "[&_[data-slot=progress-indicator]]:bg-mochi-ume",
};
