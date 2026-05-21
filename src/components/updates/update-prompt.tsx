interface UpdatePromptProps {
  version: string;
  channel: "stable" | "unstable";
  onInstall: () => void;
  onDismiss: () => void;
}

export function UpdatePrompt({ version, channel, onInstall, onDismiss }: UpdatePromptProps) {
  return (
    <section className="rounded-mochi bg-white p-4 shadow-lg ring-1 ring-slate-200">
      <p className="text-sm font-semibold">Mochi {version} is available</p>
      <p className="mt-1 text-xs text-slate-600">Channel: {channel}</p>
      <div className="mt-4 flex gap-2">
        <button
          type="button"
          className="rounded-full bg-slate-900 px-4 py-2 text-sm text-white"
          onClick={onInstall}
        >
          Update
        </button>
        <button
          type="button"
          className="rounded-full bg-slate-100 px-4 py-2 text-sm"
          onClick={onDismiss}
        >
          Later
        </button>
      </div>
    </section>
  );
}
