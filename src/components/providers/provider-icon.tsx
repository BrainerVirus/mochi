import type { ProviderId } from "@/lib/schemas/usage";

import { cn } from "@/lib/utils";

interface ProviderIconProps {
  provider: ProviderId;
  className?: string;
}

/** Monochrome provider marks for tray panel tabs (CodexBar-style branding). */
export function ProviderIcon({ provider, className }: ProviderIconProps) {
  return (
    <span
      className={cn(
        "inline-flex size-4 shrink-0 items-center justify-center text-current opacity-90",
        className,
      )}
      aria-hidden
    >
      {provider === "codex" ? <CodexMark /> : <ProviderLetter provider={provider} />}
    </span>
  );
}

function ProviderLetter({ provider }: { provider: ProviderId }) {
  const letter: Record<ProviderId, string> = {
    codex: "C",
    claude: "C",
    cursor: "U",
    gemini: "G",
    copilot: "P",
    antigravity: "A",
    factory: "F",
    zai: "Z",
    kiro: "K",
    augment: "+",
  };

  return <span className="text-[10px] leading-none font-semibold">{letter[provider]}</span>;
}

function CodexMark() {
  return (
    <svg viewBox="0 0 18 18" className="size-4" fill="currentColor" aria-hidden>
      <rect
        x="2.5"
        y="2.5"
        width="13"
        height="13"
        rx="2.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.4"
      />
      <path
        d="M8 6.5 L11 9 L8 11.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
