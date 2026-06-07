import { useId } from "react";

import { cn } from "@/lib/utils/cn";

interface MochiChibiProps {
  className?: string;
}

/** Cute mochi rice-cake blob for About and brand moments (not quota state). */
export function MochiChibi({ className }: MochiChibiProps) {
  const gradientId = useId();

  return (
    <svg viewBox="0 0 80 80" aria-hidden="true" className={cn("shrink-0", className)}>
      <defs>
        <radialGradient id={gradientId} cx="42%" cy="30%" r="68%">
          <stop offset="0%" stopColor="#FFFCF8" />
          <stop offset="55%" stopColor="#FFF8F0" />
          <stop offset="100%" stopColor="#F0E4D8" />
        </radialGradient>
      </defs>

      <ellipse cx="40" cy="58" rx="26" ry="4" fill="#334155" opacity="0.1" />

      <path
        d="M12 42c0-12.5 10.5-22 28-22s28 9.5 28 22c0 10.5-8 19-28 19s-28-8.5-28-19z"
        fill={`url(#${gradientId})`}
        stroke="#E8DED4"
        strokeWidth="1.25"
      />

      <ellipse cx="40" cy="28" rx="15" ry="6.5" fill="#FFFCF8" opacity="0.55" />

      <circle cx="26" cy="43" r="4.5" fill="#FFB5C2" opacity="0.42" />
      <circle cx="54" cy="43" r="4.5" fill="#FFB5C2" opacity="0.42" />

      <circle cx="31" cy="39" r="3" fill="#334155" />
      <circle cx="49" cy="39" r="3" fill="#334155" />
      <circle cx="32" cy="38" r="0.9" fill="#FFFCF8" opacity="0.9" />
      <circle cx="50" cy="38" r="0.9" fill="#FFFCF8" opacity="0.9" />

      <path
        d="M33.5 47c2.2 2 10.8 2 13 0"
        fill="none"
        stroke="#334155"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </svg>
  );
}
