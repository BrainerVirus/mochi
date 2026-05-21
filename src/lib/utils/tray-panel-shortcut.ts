/** macOS-style modifier label for tray panel shortcut hints. */
export function trayPanelShortcut(key: string): string {
  const isMac =
    typeof navigator !== "undefined" &&
    /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent);

  if (isMac) {
    return `⌘${key}`;
  }

  return `Ctrl+${key}`;
}
