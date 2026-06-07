const KEEP_SECTION = /^(what'?s changed|changes|fixes|features|improvements|bug fixes)$/i;
const DROP_SECTION =
  /^(install|install stable|install unstable|downloads?|assets?|binaries|packages?)$/i;
const ARTIFACT_LINE = /\.(appimage|deb|rpm|dmg|msi|exe|app\.tar\.gz)\b/i;
const COMMAND_LINE = /\b(curl|irm|bash|powershell|brew install|sudo)\b/i;

function headingTitle(line: string): string | null {
  return /^#{1,6}\s+(.+)$/.exec(line.trim())?.[1]?.trim() ?? null;
}

export function sanitizeReleaseNotesForApp(notes: string | null | undefined): string {
  if (!notes?.trim()) {
    return "";
  }

  const output: string[] = [];
  let keeping = false;
  let sawKeptHeading = false;

  for (const rawLine of notes.split("\n")) {
    const line = rawLine.trimEnd();
    const title = headingTitle(line);

    if (title) {
      if (/^mochi\s+v?\d/i.test(title)) {
        keeping = false;
        continue;
      }
      if (DROP_SECTION.test(title)) {
        keeping = false;
        continue;
      }
      if (KEEP_SECTION.test(title)) {
        keeping = true;
        sawKeptHeading = true;
        output.push(`### ${title}`);
        continue;
      }
      keeping = !sawKeptHeading;
      if (keeping) {
        output.push(line);
      }
      continue;
    }

    if (!keeping && sawKeptHeading) {
      continue;
    }
    if (ARTIFACT_LINE.test(line) || COMMAND_LINE.test(line)) {
      continue;
    }
    if (keeping || !sawKeptHeading) {
      output.push(line);
    }
  }

  return output
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}
