import type { PlatformId } from "@/lib/platform";

export function shouldRenderOverlayTitlebar(platform: PlatformId): boolean {
  return platform === "macos";
}
