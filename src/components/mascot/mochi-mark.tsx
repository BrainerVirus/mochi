import { cn } from "@/lib/utils";
import type { MochiMascotState } from "@/lib/utils/mascot-state";

interface MochiMarkProps {
  state: MochiMascotState;
  className?: string;
}

/** Accent ring colors aligned with DESIGN.md usage tokens (matcha → yuzu → ume → lavender). */
const ACCENT: Record<MochiMascotState, string> = {
  "all-good": "#A3D9A5",
  normal: "#64748B",
  warning: "#FFE4A1",
  critical: "#FF8A8A",
  "reset-soon": "#C4B5E0",
};

const ARC_OPACITY: Record<MochiMascotState, number> = {
  "all-good": 0.9,
  normal: 0.55,
  warning: 0.85,
  critical: 1,
  "reset-soon": 0.8,
};

export function MochiMark({ state, className }: MochiMarkProps) {
  const accent = ACCENT[state];
  const arcOpacity = ARC_OPACITY[state];
  const arcStrokeWidth = state === "critical" ? 2.25 : 2;

  return (
    <svg
      viewBox="0 0 32 32"
      aria-hidden="true"
      className={cn("shrink-0 text-foreground", className)}
    >
      <rect
        x="10.5"
        y="10.5"
        width="11"
        height="11"
        rx="2.75"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.25"
        opacity="0.3"
      />
      <path
        d="M16 5.5a10.5 10.5 0 1 1-9.2 18.8"
        fill="none"
        stroke={accent}
        strokeWidth={arcStrokeWidth}
        strokeLinecap="round"
        opacity={arcOpacity}
      />
      <circle cx="16" cy="16" r="2" fill="currentColor" />
    </svg>
  );
}
