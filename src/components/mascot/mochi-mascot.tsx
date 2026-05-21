import type { ReactNode } from "react";

import { cn } from "@/lib/utils";
import type { MochiMascotState } from "@/lib/utils/mascot-state";

interface MochiMascotProps {
  state: MochiMascotState;
  className?: string;
}

const INK = "#334155";

const ACCENT: Record<MochiMascotState, { ring: string; bracket: string }> = {
  "all-good": { ring: "#A3D9A5", bracket: "#A3D9A5" },
  normal: { ring: "#E7D8CC", bracket: "#FFB5C2" },
  warning: { ring: "#FFE4A1", bracket: "#FFE4A1" },
  critical: { ring: "#FF8A8A", bracket: "#FF8A8A" },
  "reset-soon": { ring: "#C4B5E0", bracket: "#C4B5E0" },
};

const FACE_BY_STATE: Record<MochiMascotState, ReactNode> = {
  "all-good": (
    <>
      <path
        d="M11.5 16.5Q12.5 15.5 13.5 16.5"
        fill="none"
        stroke={INK}
        strokeWidth="1.4"
        strokeLinecap="round"
      />
      <path
        d="M18.5 16.5Q19.5 15.5 20.5 16.5"
        fill="none"
        stroke={INK}
        strokeWidth="1.4"
        strokeLinecap="round"
      />
      <path
        d="M11.5 20.5Q16 23.5 20.5 20.5"
        fill="none"
        stroke={INK}
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </>
  ),
  warning: (
    <>
      <ellipse cx="12.5" cy="17" rx="1.4" ry="1.8" fill={INK} />
      <ellipse cx="19.5" cy="17" rx="1.4" ry="1.8" fill={INK} />
      <path
        d="M12 21.5Q16 20 20 21.5"
        fill="none"
        stroke={INK}
        strokeWidth="1.4"
        strokeLinecap="round"
      />
    </>
  ),
  critical: (
    <>
      <path d="M10.5 16.5L14 19.5" stroke={INK} strokeWidth="1.4" strokeLinecap="round" />
      <path d="M14 16.5L10.5 19.5" stroke={INK} strokeWidth="1.4" strokeLinecap="round" />
      <path d="M18 16.5L21.5 19.5" stroke={INK} strokeWidth="1.4" strokeLinecap="round" />
      <path d="M21.5 16.5L18 19.5" stroke={INK} strokeWidth="1.4" strokeLinecap="round" />
      <path d="M11.5 22.5h9" stroke={INK} strokeWidth="1.5" strokeLinecap="round" />
    </>
  ),
  "reset-soon": (
    <>
      <path
        d="M11.5 17l1 1.5 1-1.5"
        fill="none"
        stroke={INK}
        strokeWidth="1.2"
        strokeLinecap="round"
      />
      <path
        d="M18.5 17l1 1.5 1-1.5"
        fill="none"
        stroke={INK}
        strokeWidth="1.2"
        strokeLinecap="round"
      />
      <ellipse cx="16" cy="21.5" rx="2.2" ry="2.5" fill="none" stroke={INK} strokeWidth="1.3" />
    </>
  ),
  normal: (
    <>
      <circle cx="12.5" cy="17" r="1.2" fill={INK} />
      <circle cx="19.5" cy="17" r="1.2" fill={INK} />
      <path
        d="M13 20.5Q16 22 19 20.5"
        fill="none"
        stroke={INK}
        strokeWidth="1.4"
        strokeLinecap="round"
      />
      <path
        d="M14.5 19.8h3"
        stroke="#64748B"
        strokeWidth="0.9"
        strokeLinecap="round"
        opacity="0.55"
      />
    </>
  ),
};

export function MochiMascot({ state, className }: MochiMascotProps) {
  const accent = ACCENT[state];
  const squished = state === "critical";

  return (
    <svg viewBox="0 0 32 32" aria-hidden="true" className={cn("size-24 shrink-0", className)}>
      <path
        d="M5.5 11.5Q3.5 16 5.5 20.5"
        fill="none"
        stroke={accent.bracket}
        strokeWidth="1.6"
        strokeLinecap="round"
        opacity="0.85"
      />
      <path
        d="M26.5 11.5Q28.5 16 26.5 20.5"
        fill="none"
        stroke={accent.bracket}
        strokeWidth="1.6"
        strokeLinecap="round"
        opacity="0.85"
      />

      <rect x="14.25" y="5.5" width="3.5" height="2.5" rx="0.6" fill="#C4B5E0" opacity="0.9" />
      <circle cx="16" cy="6.75" r="0.45" fill="#FFF8F0" />

      <ellipse
        cx="16"
        cy={squished ? 18.5 : 17.5}
        rx={squished ? 11.5 : 11}
        ry={squished ? 9.5 : 11.5}
        fill="#FFF8F0"
        stroke={accent.ring}
        strokeWidth="1.5"
      />

      {FACE_BY_STATE[state]}

      {state === "all-good" ? (
        <>
          <path
            d="M22 9.5l0.6 1.2 1.3 0.2-0.95 0.9 0.25 1.25-1.15-0.6-1.15 0.6 0.25-1.25-0.95-0.9 1.3-0.2z"
            fill="#A3D9A5"
            opacity="0.9"
          />
          <rect x="12.5" y="7.5" width="4" height="0.8" rx="0.4" fill="#64748B" opacity="0.45" />
        </>
      ) : null}

      {state === "reset-soon" ? (
        <g stroke="#C4B5E0" strokeWidth="0.8" fill="none">
          <circle cx="24.5" cy="9.5" r="2.6" />
          <path d="M24.5 9.5V7.8" strokeLinecap="round" />
          <path d="M24.5 9.5l1.1 0.7" strokeLinecap="round" />
        </g>
      ) : null}

      {state === "warning" || state === "critical" ? (
        <>
          <ellipse cx="24.5" cy="13.5" rx="0.9" ry="1.3" fill="#93C5FD" opacity="0.75" />
          {state === "critical" ? (
            <ellipse cx="7.5" cy="13.5" rx="0.9" ry="1.3" fill="#93C5FD" opacity="0.75" />
          ) : null}
        </>
      ) : null}
    </svg>
  );
}
