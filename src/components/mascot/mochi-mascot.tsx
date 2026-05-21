interface MochiMascotProps {
  state: "normal" | "warning" | "critical" | "reset-soon" | "all-good";
}

export function MochiMascot({ state }: MochiMascotProps) {
  const blush = state === "critical" ? "#FF8A8A" : "#FFB5C2";

  return (
    <svg viewBox="0 0 120 100" aria-hidden="true" className="size-24">
      <ellipse cx="60" cy="56" rx="42" ry="34" fill="#FFF8F0" stroke="#E7D8CC" strokeWidth="3" />
      <circle cx="45" cy="52" r="4" fill="#1F2937" />
      <circle cx="75" cy="52" r="4" fill="#1F2937" />
      <circle cx="36" cy="62" r="6" fill={blush} opacity="0.6" />
      <circle cx="84" cy="62" r="6" fill={blush} opacity="0.6" />
      <path
        d="M51 66 Q60 72 69 66"
        fill="none"
        stroke="#1F2937"
        strokeWidth="3"
        strokeLinecap="round"
      />
    </svg>
  );
}
