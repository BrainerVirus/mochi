export function formatUpdatedAgo(updatedAt: string, now = new Date()): string {
  const date = new Date(updatedAt);
  if (Number.isNaN(date.getTime())) {
    return `Updated ${updatedAt}`;
  }

  const deltaMs = now.getTime() - date.getTime();
  if (deltaMs < 60_000) {
    return "Updated just now";
  }

  const totalMinutes = Math.floor(deltaMs / 60_000);
  if (totalMinutes < 60) {
    return `Updated ${totalMinutes}m ago`;
  }

  const totalHours = Math.floor(totalMinutes / 60);
  if (totalHours < 24) {
    return `Updated ${totalHours}h ago`;
  }

  return `Updated ${date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  })}`;
}
