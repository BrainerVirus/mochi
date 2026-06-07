import { describe, expect, it, vi } from "vitest";

import { observeSegmentTrackResize } from "@/features/tray/components/segment-track-resize-observer";

function mockTrack(): HTMLElement {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- test double without DOM
  return { tagName: "DIV" } as HTMLElement;
}

let resizeHandler: ResizeObserverCallback = () => {};

class MockResizeObserver {
  constructor(callback: ResizeObserverCallback) {
    resizeHandler = callback;
  }

  observe() {}

  disconnect() {}
}

describe("observeSegmentTrackResize", () => {
  it("debounces resize callbacks to one rAF per frame", () => {
    const rafCallbacks = new Map<number, FrameRequestCallback>();
    let nextFrameId = 0;

    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
      const frameId = ++nextFrameId;
      rafCallbacks.set(frameId, callback);
      return frameId;
    });
    vi.stubGlobal("cancelAnimationFrame", (frameId: number) => {
      rafCallbacks.delete(frameId);
    });
    vi.stubGlobal("ResizeObserver", MockResizeObserver);

    const track = mockTrack();
    const onResize = vi.fn<() => void>();

    observeSegmentTrackResize(track, onResize);
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mock ResizeObserver entry
    resizeHandler([], {} as ResizeObserver);
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mock ResizeObserver entry
    resizeHandler([], {} as ResizeObserver);

    expect(onResize).not.toHaveBeenCalled();
    expect(rafCallbacks.size).toBe(1);

    const pending = [...rafCallbacks.values()][0];
    pending?.(0);
    expect(onResize).toHaveBeenCalledOnce();

    vi.unstubAllGlobals();
  });
});
