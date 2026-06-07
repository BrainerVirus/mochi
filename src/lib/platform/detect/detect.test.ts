import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn<(command: string) => Promise<unknown>>(),
}));

import { invoke } from "@tauri-apps/api/core";

import { detectPlatformFromNavigator, parsePlatformId } from "@/lib/platform/detect";

describe("parsePlatformId", () => {
  it("accepts known desktop platforms", () => {
    expect(parsePlatformId("macos")).toBe("macos");
    expect(parsePlatformId("windows")).toBe("windows");
    expect(parsePlatformId("linux")).toBe("linux");
  });

  it("falls back to unknown for unexpected values", () => {
    expect(parsePlatformId("android")).toBe("unknown");
  });
});

describe("detectPlatformFromNavigator", () => {
  it("returns unknown when navigator is unavailable", () => {
    const navigatorRef = globalThis.navigator;
    // @ts-expect-error — simulate SSR
    delete globalThis.navigator;

    expect(detectPlatformFromNavigator()).toBe("unknown");

    globalThis.navigator = navigatorRef;
  });
});

describe("detectPlatform invoke mapping", () => {
  it("maps Tauri platform strings through parsePlatformId", async () => {
    vi.mocked(invoke).mockResolvedValueOnce("linux");

    const runtime = await import("@/lib/tauri/runtime");
    const detect = await import("@/lib/platform/detect");
    const isTauriSpy = vi.spyOn(runtime, "isTauriRuntime").mockReturnValue(true);

    await expect(detect.detectPlatform()).resolves.toBe("linux");
    expect(invoke).toHaveBeenCalledWith("get_platform");

    isTauriSpy.mockRestore();
  });
});
