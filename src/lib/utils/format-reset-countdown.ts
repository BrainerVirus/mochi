const RESETTING_LABEL = "Resetting…";

export function formatResetCountdown(resetsAt: string | null, now = new Date()): string | null {
  if (!resetsAt) {
    return null;
  }

  const reset = new Date(resetsAt);
  if (Number.isNaN(reset.getTime())) {
    return null;
  }

  const diffMs = reset.getTime() - now.getTime();
  if (diffMs <= 0) {
    return RESETTING_LABEL;
  }

  const totalMinutes = Math.floor(diffMs / 60_000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours >= 24) {
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h`;
  }

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  return `${minutes}m`;
}
