import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  cacheReleaseNotes,
  readCachedReleaseNotes,
  RELEASE_NOTES_CACHE_KEY,
} from "./release-notes-cache";

function createStorageMock(): Storage {
  const store = new Map<string, string>();
  return {
    get length() {
      return store.size;
    },
    clear() {
      store.clear();
    },
    getItem(key: string) {
      return store.get(key) ?? null;
    },
    key(index: number) {
      return [...store.keys()][index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(key, value);
    },
  };
}

describe("release-notes-cache", () => {
  beforeEach(() => {
    vi.stubGlobal("localStorage", createStorageMock());
    vi.stubGlobal("sessionStorage", createStorageMock());
  });

  it("stores and reads cached release notes", () => {
    cacheReleaseNotes({
      version: "0.2.0",
      notes: "## Features\n- Updater UX",
      channel: "stable",
      cachedAt: "2026-05-24T12:00:00.000Z",
    });

    expect(readCachedReleaseNotes()).toEqual({
      version: "0.2.0",
      notes: "## Features\n- Updater UX",
      channel: "stable",
      cachedAt: "2026-05-24T12:00:00.000Z",
    });
  });

  it("returns null for invalid cache payloads", () => {
    localStorage.setItem(RELEASE_NOTES_CACHE_KEY, "{not-json");
    expect(readCachedReleaseNotes()).toBeNull();
  });
});
