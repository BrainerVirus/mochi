function formatResetClock(date: Date): string {
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  return `${hours}:${minutes}`;
}

/**
 * CodexBar-style reset label for tray meter rows, e.g. "Resets 18:24" or "Resets May 26".
 */
export function formatResetLine(resetsAt: string | null, now = new Date()): string | null {
  if (!resetsAt) {
    return null;
  }

  const reset = new Date(resetsAt);
  if (Number.isNaN(reset.getTime())) {
    return null;
  }

  const diffMs = reset.getTime() - now.getTime();
  if (diffMs <= 0) {
    return "Resets now";
  }

  if (reset.toDateString() === now.toDateString()) {
    return `Resets ${formatResetClock(reset)}`;
  }

  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  if (reset.toDateString() === tomorrow.toDateString()) {
    return `Resets tomorrow, ${formatResetClock(reset)}`;
  }

  const dateLabel = reset.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });

  return `Resets ${dateLabel}, ${formatResetClock(reset)}`;
}
