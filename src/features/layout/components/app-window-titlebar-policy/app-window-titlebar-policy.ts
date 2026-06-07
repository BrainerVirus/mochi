import type { PlatformId } from "@/lib/platform/types";

export function shouldRenderOverlayTitlebar(platform: PlatformId): boolean {
  return platform === "macos";
}
