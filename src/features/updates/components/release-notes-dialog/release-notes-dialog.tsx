"use client";

import { useMemo } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollFadeRegion } from "@/features/tray/components/scroll-fade-region";
import { splitPatchNotesSections, type PatchNotesSection } from "@/lib/updates/format-patch-notes";

interface ReleaseNotesDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  version: string | null;
  channel: string;
  notesDescription?: string;
  notes: string | null;
  isChecking: boolean;
  onRecheck: () => void;
}

export function ReleaseNotesDialog({
  open,
  onOpenChange,
  version,
  channel,
  notesDescription,
  notes,
  isChecking,
  onRecheck,
}: ReleaseNotesDialogProps) {
  const sections = useMemo(() => splitPatchNotesSections(notes), [notes]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="app-window-dialog flex max-h-[min(32rem,85vh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-md">
        <DialogHeader className="shrink-0 border-b px-4 pt-4 pb-3 text-left">
          <DialogTitle>What&apos;s new</DialogTitle>
          <DialogDescription>
            {notesDescription ?? (version ? `Version ${version}` : "Release notes")} · Channel:{" "}
            {channel}
          </DialogDescription>
        </DialogHeader>

        <ScrollFadeRegion
          orientation="vertical"
          controls="none"
          className="min-h-0 flex-1"
          scrollClassName="overscroll-y-contain px-4 py-3"
        >
          <PatchNotesSections sections={sections} />
          {sections.length === 0 ? (
            <p className="text-muted-foreground text-xs leading-relaxed">
              {isChecking
                ? "Checking for updates…"
                : "No release notes cached yet. Check for updates to fetch the latest notes."}
            </p>
          ) : null}
        </ScrollFadeRegion>

        <DialogFooter className="shrink-0 border-t px-4 py-3">
          <Button
            type="button"
            className="w-full"
            disabled={isChecking}
            onClick={() => {
              onRecheck();
            }}
          >
            {isChecking ? "Checking…" : "Check for updates"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function PatchNotesSections({ sections }: { sections: PatchNotesSection[] }) {
  if (sections.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-col gap-4">
      {sections.map((section) => (
        <section key={section.title ?? section.items.join("-")}>
          {section.title ? (
            <h2 className="text-foreground mb-1.5 text-xs font-semibold">{section.title}</h2>
          ) : null}
          <ul className="text-muted-foreground list-disc space-y-1 pl-4 text-xs leading-relaxed">
            {section.items.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
