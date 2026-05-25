import { useEffect } from "react";

import {
  installDiagnosticsHandlers,
  reportFrontendBoot,
} from "@/lib/tauri/diagnostics";

/** Registers global error handlers and reports one boot snapshot per webview. */
export function useDiagnosticsBoot() {
  useEffect(() => {
    installDiagnosticsHandlers();
    void reportFrontendBoot();
  }, []);
}
