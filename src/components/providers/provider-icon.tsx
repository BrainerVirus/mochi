import type { ProviderId } from "@/lib/schemas/usage";

import { cn } from "@/lib/utils";

import { PROVIDER_BRAND_SVGS } from "./provider-icon-sources";

interface ProviderIconProps {
  provider: ProviderId;
  className?: string;
}

/** Monochrome provider brand marks for tray tabs, overview rows, and usage cards. */
export function ProviderIcon({ provider, className }: ProviderIconProps) {
  const markup = PROVIDER_BRAND_SVGS[provider];

  if (!markup) {
    return <ProviderLetter provider={provider} className={className} />;
  }

  return (
    <span
      className={cn(
        "inline-flex size-4 shrink-0 items-center justify-center text-current opacity-90 [&>svg]:size-full",
        className,
      )}
      aria-hidden
      dangerouslySetInnerHTML={{ __html: markup }}
    />
  );
}

function ProviderLetter({ provider, className }: { provider: ProviderId; className?: string }) {
  const letter: Record<ProviderId, string> = {
    codex: "C",
    claude: "C",
    cursor: "U",
    gemini: "G",
    copilot: "P",
    opencode: "O",
    "opencode-go": "G",
    antigravity: "A",
    factory: "F",
    zai: "Z",
    kiro: "K",
    augment: "+",
  };

  return (
    <span
      className={cn(
        "inline-flex size-4 shrink-0 items-center justify-center text-current opacity-90",
        className,
      )}
      aria-hidden
    >
      <span className="text-[10px] leading-none font-semibold">{letter[provider]}</span>
    </span>
  );
}
