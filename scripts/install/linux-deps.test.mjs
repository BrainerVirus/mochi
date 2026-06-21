import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { bashSyntaxCheck, bashBin, sourceLinuxDeps } from "./test-helpers.mjs";

const linuxDepsSh = path.join(import.meta.dirname, "lib/linux-deps.sh");

describe("linux-deps.sh", () => {
  it(`parses under ${bashBin()}`, () => {
    expect(() => bashSyntaxCheck(linuxDepsSh)).not.toThrow();
  });

  it("detects debian, fedora, and arch families from os-release fixtures", () => {
    const dir = mkdtempSync(path.join(tmpdir(), "mochi-os-release-"));
    const debian = path.join(dir, "debian");
    writeFileSync(debian, "ID=ubuntu\nID_LIKE=debian\n");
    const fedora = path.join(dir, "fedora");
    writeFileSync(fedora, 'ID=fedora\nID_LIKE="fedora rhel"\n');
    const arch = path.join(dir, "arch");
    writeFileSync(arch, "ID=arch\nID_LIKE=arch\n");

    expect(sourceLinuxDeps(`mochi_detect_linux_family`, { MOCHI_TEST_OS_RELEASE: debian })).toBe(
      "debian",
    );
    expect(sourceLinuxDeps(`mochi_detect_linux_family`, { MOCHI_TEST_OS_RELEASE: fedora })).toBe(
      "fedora",
    );
    expect(sourceLinuxDeps(`mochi_detect_linux_family`, { MOCHI_TEST_OS_RELEASE: arch })).toBe(
      "arch",
    );
  });

  it("skips GNOME tray extension setup when MOCHI_GNOME_TRAY=0", () => {
    const output = sourceLinuxDeps(`XDG_CURRENT_DESKTOP=GNOME mochi_ensure_gnome_tray_extension`, {
      MOCHI_GNOME_TRAY: "0",
    });
    expect(output).toContain("Skipping GNOME tray extension setup (MOCHI_GNOME_TRAY=0)");
  });

  it("does not attempt GNOME extension work outside GNOME desktops", () => {
    const output = sourceLinuxDeps(`XDG_CURRENT_DESKTOP=KDE mochi_ensure_gnome_tray_extension`, {
      MOCHI_GNOME_TRAY: "1",
    });
    expect(output).toBe("");
  });
});
