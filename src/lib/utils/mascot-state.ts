import type { UsageSnapshot } from "@/lib/schemas/usage";

import { getUsageMeterTone } from "./usage-meter-tone";

export type MochiMascotState = "normal" | "warning" | "critical" | "reset-soon" | "all-good";

export function getMascotStateFromSnapshots(
  snapshots: UsageSnapshot[],
  options: { isError?: boolean } = {},
): MochiMascotState {
  if (options.isError) {
    return "warning";
  }

  if (snapshots.length === 0) {
    return "normal";
  }

  const highestUsedPercent = Math.max(...snapshots.map((snapshot) => snapshot.primary.used_percent));
  const tone = getUsageMeterTone(highestUsedPercent);

  if (tone === "critical") {
    return "critical";
  }

  if (tone === "warning") {
    return "warning";
  }

  if (highestUsedPercent < 20) {
    return "all-good";
  }

  return "normal";
}
