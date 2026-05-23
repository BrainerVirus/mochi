import type { ProviderId } from "@/lib/schemas/usage";

import antigravitySvg from "@/assets/providers/antigravity.svg?raw";
import augmentSvg from "@/assets/providers/augment.svg?raw";
import claudeSvg from "@/assets/providers/claude.svg?raw";
import codexSvg from "@/assets/providers/codex.svg?raw";
import copilotSvg from "@/assets/providers/copilot.svg?raw";
import cursorSvg from "@/assets/providers/cursor.svg?raw";
import factorySvg from "@/assets/providers/factory.svg?raw";
import geminiSvg from "@/assets/providers/gemini.svg?raw";
import kiroSvg from "@/assets/providers/kiro.svg?raw";
import opencodeGoSvg from "@/assets/providers/opencode-go.svg?raw";
import opencodeSvg from "@/assets/providers/opencode.svg?raw";
import zaiSvg from "@/assets/providers/zai.svg?raw";

/** Bundled monochrome brand marks (CodexBar-derived), keyed by provider id. */
export const PROVIDER_BRAND_SVGS: Record<ProviderId, string> = {
  codex: codexSvg,
  claude: claudeSvg,
  cursor: cursorSvg,
  gemini: geminiSvg,
  copilot: copilotSvg,
  opencode: opencodeSvg,
  "opencode-go": opencodeGoSvg,
  antigravity: antigravitySvg,
  factory: factorySvg,
  zai: zaiSvg,
  kiro: kiroSvg,
  augment: augmentSvg,
};
