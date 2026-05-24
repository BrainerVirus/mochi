export interface PatchNotesSection {
  title: string | null;
  items: string[];
}

function normalizeLine(line: string): string | null {
  const trimmed = line.trim();
  if (!trimmed) {
    return null;
  }

  return trimmed.replace(/^[-*]\s+/, "");
}

export function formatPatchNotes(notes: string | null | undefined): string[] {
  if (!notes?.trim()) {
    return [];
  }

  const blocks = notes.split(/\n{2,}/);
  const lines: string[] = [];

  for (const block of blocks) {
    const normalized = block
      .split("\n")
      .map(normalizeLine)
      .filter((line): line is string => line !== null);

    lines.push(...normalized);
  }

  return lines;
}

export function splitPatchNotesSections(notes: string | null | undefined): PatchNotesSection[] {
  if (!notes?.trim()) {
    return [];
  }

  const sections: PatchNotesSection[] = [];
  let current: PatchNotesSection = { title: null, items: [] };

  for (const rawLine of notes.split("\n")) {
    const line = rawLine.trim();
    if (!line) {
      continue;
    }

    const headingMatch = /^#{1,6}\s+(.+)$/.exec(line);
    if (headingMatch) {
      if (current.title !== null || current.items.length > 0) {
        sections.push(current);
      }
      current = { title: headingMatch[1] ?? null, items: [] };
      continue;
    }

    const normalized = normalizeLine(line);
    if (normalized) {
      current.items.push(normalized);
    }
  }

  if (current.title !== null || current.items.length > 0) {
    sections.push(current);
  }

  return sections;
}
