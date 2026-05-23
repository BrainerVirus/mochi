import { invoke } from "@tauri-apps/api/core";

import { PlatformIdSchema, type PlatformId } from "@/lib/platform/types";
import { isTauriRuntime } from "@/lib/tauri/runtime";

export function parsePlatformId(value: string): PlatformId {
  const parsed = PlatformIdSchema.safeParse(value);
  return parsed.success ? parsed.data : "unknown";
}

/** Best-effort platform guess for browser / Vitest without Tauri. */
export function detectPlatformFromNavigator(): PlatformId {
  if (typeof navigator === "undefined") {
    return "unknown";
  }

  const platform = navigator.platform.toLowerCase();
  const userAgent = navigator.userAgent.toLowerCase();

  if (platform.includes("mac") || userAgent.includes("mac os")) {
    return "macos";
  }

  if (platform.includes("win") || userAgent.includes("windows")) {
    return "windows";
  }

  if (platform.includes("linux") || userAgent.includes("linux")) {
    return "linux";
  }

  return "unknown";
}

export async function detectPlatform(): Promise<PlatformId> {
  if (!isTauriRuntime()) {
    return detectPlatformFromNavigator();
  }

  try {
    const result = await invoke<string>("get_platform");
    return parsePlatformId(result);
  } catch {
    return detectPlatformFromNavigator();
  }
}
