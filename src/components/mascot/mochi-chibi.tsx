import { cn } from "@/lib/utils";

interface MochiChibiProps {
  className?: string;
}

/** Cute chibi mochi rice-cake mascot for About and brand moments (not quota state). */
export function MochiChibi({ className }: MochiChibiProps) {
  return (
    <svg viewBox="0 0 80 80" aria-hidden="true" className={cn("shrink-0", className)}>
      <ellipse cx="40" cy="68" rx="22" ry="4.5" fill="#334155" opacity="0.12" />
      <path
        d="M18 44c0-14.4 9.8-26 22-26s22 11.6 22 26c0 8.2-4.2 15.4-10.5 19.6-3.4 2.4-7.4 3.6-11.5 3.6s-8.1-1.2-11.5-3.6C22.2 59.4 18 52.2 18 44z"
        fill="#FFF8F0"
        stroke="#E8DED4"
        strokeWidth="1.25"
      />
      <path
        d="M26 30c6-8 22-8 28 0"
        fill="none"
        stroke="#F5EDE4"
        strokeWidth="2"
        strokeLinecap="round"
        opacity="0.85"
      />
      <ellipse cx="40" cy="28" rx="16" ry="7" fill="#FFFCF8" opacity="0.55" />
      <circle cx="30" cy="44" r="5.5" fill="#FFB5C2" opacity="0.45" />
      <circle cx="50" cy="44" r="5.5" fill="#FFB5C2" opacity="0.45" />
      <circle cx="31" cy="38" r="3.25" fill="#334155" />
      <circle cx="49" cy="38" r="3.25" fill="#334155" />
      <circle cx="32.1" cy="36.9" r="1" fill="#FFFCF8" opacity="0.9" />
      <circle cx="50.1" cy="36.9" r="1" fill="#FFFCF8" opacity="0.9" />
      <path
        d="M34 47.5c2.2 2.4 9.8 2.4 12 0"
        fill="none"
        stroke="#334155"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
      <ellipse cx="22" cy="50" rx="4" ry="5.5" fill="#FFF8F0" stroke="#E8DED4" strokeWidth="1" />
      <ellipse cx="58" cy="50" rx="4" ry="5.5" fill="#FFF8F0" stroke="#E8DED4" strokeWidth="1" />
      <ellipse cx="32" cy="62" rx="5" ry="3.5" fill="#FFF8F0" stroke="#E8DED4" strokeWidth="1" />
      <ellipse cx="48" cy="62" rx="5" ry="3.5" fill="#FFF8F0" stroke="#E8DED4" strokeWidth="1" />
    </svg>
  );
}
