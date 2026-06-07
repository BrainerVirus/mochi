import { describe, expect, it } from "vitest";

import { formatPatchNotes, splitPatchNotesSections } from "./format-patch-notes";

describe("formatPatchNotes", () => {
  it("returns empty array for blank notes", () => {
    expect(formatPatchNotes(null)).toEqual([]);
    expect(formatPatchNotes("   ")).toEqual([]);
  });

  it("normalizes bullet lines and trims whitespace", () => {
    const notes = "- Fix tray scroll\n  - Improve settings\n* Add updater";
    expect(formatPatchNotes(notes)).toEqual(["Fix tray scroll", "Improve settings", "Add updater"]);
  });

  it("preserves paragraph breaks as separate blocks", () => {
    const notes = "First paragraph.\n\nSecond paragraph.";
    expect(formatPatchNotes(notes)).toEqual(["First paragraph.", "Second paragraph."]);
  });
});

describe("splitPatchNotesSections", () => {
  it("groups lines under markdown headings", () => {
    const notes = "## Features\n- Tray update button\n\n## Fixes\n- Scroll bug";
    expect(splitPatchNotesSections(notes)).toEqual([
      { title: "Features", items: ["Tray update button"] },
      { title: "Fixes", items: ["Scroll bug"] },
    ]);
  });
});
