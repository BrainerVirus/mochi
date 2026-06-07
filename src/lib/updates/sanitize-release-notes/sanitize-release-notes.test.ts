import { describe, expect, it } from "vitest";

import { sanitizeReleaseNotesForApp } from "./sanitize-release-notes";

describe("sanitizeReleaseNotesForApp", () => {
  it("keeps change sections and removes install commands", () => {
    const notes = [
      "## Mochi v0.2.0",
      "",
      "### What's changed",
      "- SQLite usage persistence",
      "- Cached-usage CLI",
      "",
      "### Install stable",
      "- macOS: `curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos.sh | bash`",
      "- Linux: `curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash`",
    ].join("\n");

    expect(sanitizeReleaseNotesForApp(notes)).toBe(
      "### What's changed\n- SQLite usage persistence\n- Cached-usage CLI",
    );
  });

  it("drops artifact lists and binary file names", () => {
    const notes = [
      "### Fixes",
      "- Fix tray scrolling",
      "",
      "### Assets",
      "- Mochi_0.2.0_amd64.AppImage",
      "- Mochi_0.2.0_x64.dmg",
    ].join("\n");

    expect(sanitizeReleaseNotesForApp(notes)).toBe("### Fixes\n- Fix tray scrolling");
  });
});
