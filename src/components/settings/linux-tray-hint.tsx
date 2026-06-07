import { useEffect, useState } from "react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { detectPlatform } from "@/lib/platform/detect";
import type { PlatformId } from "@/lib/platform/types";

export function LinuxTrayHint() {
  const [platform, setPlatform] = useState<PlatformId>("unknown");

  useEffect(() => {
    void detectPlatform().then(setPlatform);
  }, []);

  if (platform !== "linux") {
    return null;
  }

  return (
    <Alert className="mb-4" data-linux-tray-hint>
      <AlertTitle>Linux tray</AlertTitle>
      <AlertDescription>
        On GNOME, install an AppIndicator extension and log out and back in if the tray icon is
        missing. Use the desktop widget or{" "}
        <a
          href="https://github.com/BrainerVirus/mochi/blob/main/docs/linux.md"
          className="underline underline-offset-2"
          target="_blank"
          rel="noreferrer"
        >
          status-bar mode
        </a>{" "}
        as a fallback.
      </AlertDescription>
    </Alert>
  );
}
