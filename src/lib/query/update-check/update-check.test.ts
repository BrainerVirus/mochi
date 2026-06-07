import { QueryClient, type QueryFunctionContext } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/tauri/commands", () => ({
  checkForUpdate: vi.fn<(channel: string) => Promise<unknown>>(),
}));

vi.mock("@/lib/updates/current-release-notes", () => ({
  fetchCurrentReleaseNotes: vi.fn<(channel: string) => Promise<unknown>>(),
}));

vi.mock("@/lib/updates/release-notes-cache", () => ({
  cacheReleaseNotes: vi.fn<(entry: unknown) => void>(),
}));

import { checkForUpdate } from "@/lib/tauri/commands";
import { fetchCurrentReleaseNotes } from "@/lib/updates/current-release-notes";
import { cacheReleaseNotes } from "@/lib/updates/release-notes-cache";

import { createUpdateCheckQueryOptions } from "./update-check";

describe("createUpdateCheckQueryOptions", () => {
  const queryClient = new QueryClient();

  beforeEach(() => {
    vi.mocked(checkForUpdate).mockReset();
    vi.mocked(fetchCurrentReleaseNotes).mockReset();
    vi.mocked(cacheReleaseNotes).mockReset();
  });

  it("fetches current release notes when updater check fails", async () => {
    vi.mocked(checkForUpdate).mockRejectedValue(new Error("updater failed"));
    vi.mocked(fetchCurrentReleaseNotes).mockResolvedValue(null);

    const options = createUpdateCheckQueryOptions("stable");

    await expect(runUpdateCheckQuery(options.queryFn, "stable", queryClient)).rejects.toThrow(
      "updater failed",
    );
    expect(fetchCurrentReleaseNotes).toHaveBeenCalledWith("stable");
  });

  it("caches notes from updater manifests", async () => {
    vi.mocked(checkForUpdate).mockResolvedValue({
      available: true,
      version: "0.1.6",
      channel: "unstable",
      notes:
        "### What's changed\n- Current release only\n\n### Install stable\n- macOS: `curl example`",
    });

    const options = createUpdateCheckQueryOptions("unstable");

    await expect(
      runUpdateCheckQuery(options.queryFn, "unstable", queryClient),
    ).resolves.toMatchObject({
      available: true,
    });
    expect(cacheReleaseNotes).toHaveBeenCalledWith(
      expect.objectContaining({
        version: "0.1.6",
        channel: "unstable",
        notes: "### What's changed\n- Current release only",
        source: "updater",
      }),
    );
  });
});

function runUpdateCheckQuery(
  queryFn:
    | ((context: QueryFunctionContext<readonly ["update", "check", string]>) => unknown)
    | undefined,
  channel: "stable" | "unstable",
  client: QueryClient,
) {
  if (!queryFn) {
    throw new Error("missing query function");
  }

  return queryFn({
    client,
    queryKey: ["update", "check", channel],
    signal: new AbortController().signal,
    meta: undefined,
  });
}
