import type { UsageWindow } from "@/lib/schemas/usage";

/** Default window length when resets_at is known but duration is not (weekly). */
const DEFAULT_WINDOW_MINUTES = 10_080;

export type UsagePaceStage =
  | "on-track"
  | "slightly-ahead"
  | "ahead"
  | "far-ahead"
  | "slightly-behind"
  | "behind"
  | "far-behind";

export interface UsagePaceDetail {
  leftLabel: string;
  rightLabel: string | null;
  expectedUsedPercent: number;
  stage: UsagePaceStage;
}

function windowMinutes(window: UsageWindow): number | null {
  const label = window.label.toLowerCase();
  if (label.includes("session") || label.includes("5-hour") || label.includes("5 hour")) {
    return 300;
  }
  if (label.includes("daily") || label.includes("24")) {
    return 1_440;
  }
  if (label.includes("weekly") || label.includes("7-day") || label.includes("7 day")) {
    return 10_080;
  }
  if (label.includes("monthly") || label.includes("30 day")) {
    return 43_200;
  }
  return null;
}

function paceStage(delta: number): UsagePaceStage {
  const abs = Math.abs(delta);
  if (abs <= 2) return "on-track";
  if (abs <= 6) return delta >= 0 ? "slightly-ahead" : "slightly-behind";
  if (abs <= 12) return delta >= 0 ? "ahead" : "behind";
  return delta >= 0 ? "far-ahead" : "far-behind";
}

function leftLabelForStage(stage: UsagePaceStage, delta: number): string {
  const deltaValue = Math.round(Math.abs(delta));
  switch (stage) {
    case "on-track":
      return "On pace";
    case "slightly-ahead":
    case "ahead":
    case "far-ahead":
      return `${deltaValue}% ahead`;
    default:
      return `${deltaValue}% in reserve`;
  }
}

/**
 * Linear pace projection (CodexBar `UsagePace.weekly` subset).
 * Returns reserve/pace copy when resets_at and window duration are known.
 */
export function usagePaceDetail(
  window: UsageWindow,
  now: Date = new Date(),
): UsagePaceDetail | null {
  if (window.remaining_percent <= 0) {
    return null;
  }

  const resetsAt = window.resets_at;
  if (!resetsAt) {
    return null;
  }

  const resetDate = new Date(resetsAt);
  const timeUntilResetMs = resetDate.getTime() - now.getTime();
  if (timeUntilResetMs <= 0) {
    return null;
  }

  const minutes = windowMinutes(window) ?? DEFAULT_WINDOW_MINUTES;
  const durationMs = minutes * 60 * 1000;
  if (timeUntilResetMs > durationMs) {
    return null;
  }

  const elapsedMs = durationMs - timeUntilResetMs;
  const expectedUsed = Math.min(100, Math.max(0, (elapsedMs / durationMs) * 100));
  const actualUsed = window.used_percent;
  if (elapsedMs === 0 && actualUsed > 0) {
    return null;
  }

  const delta = actualUsed - expectedUsed;
  const stage = paceStage(delta);

  let rightLabel: string | null = null;
  if (elapsedMs > 0 && actualUsed > 0) {
    const rate = actualUsed / elapsedMs;
    if (rate > 0) {
      const remaining = Math.max(0, 100 - actualUsed);
      const etaMs = (remaining / rate) * elapsedMs;
      if (etaMs >= timeUntilResetMs) {
        rightLabel = "Lasts until reset";
      }
    }
  } else if (elapsedMs > 0 && actualUsed === 0) {
    rightLabel = "Lasts until reset";
  }

  return {
    leftLabel: leftLabelForStage(stage, delta),
    rightLabel,
    expectedUsedPercent: expectedUsed,
    stage,
  };
}

export function reserveDetailLeft(window: UsageWindow): string | null {
  const pace = usagePaceDetail(window);
  if (pace) {
    return pace.leftLabel;
  }
  if (window.remaining_percent > 0) {
    return `${Math.round(window.remaining_percent)}% in reserve`;
  }
  return null;
}

export function reserveDetailRight(window: UsageWindow): string | null {
  const pace = usagePaceDetail(window);
  if (pace?.rightLabel) {
    return pace.rightLabel;
  }
  return window.resets_at ? "Lasts until reset" : null;
}
