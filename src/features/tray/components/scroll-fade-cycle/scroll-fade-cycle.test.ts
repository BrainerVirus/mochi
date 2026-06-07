import { describe, expect, it } from "vitest";

import {
  cycleVerticalScrollBackward,
  cycleVerticalScrollForward,
} from "@/features/tray/components/scroll-fade-cycle";

function mockScrollContainer({
  scrollTop,
  scrollHeight,
  clientHeight,
}: {
  scrollTop: number;
  scrollHeight: number;
  clientHeight: number;
}) {
  const scrollToCalls: ScrollToOptions[] = [];
  const container = {
    scrollTop,
    scrollHeight,
    clientHeight,
    scrollTo(options: ScrollToOptions) {
      scrollToCalls.push(options);
    },
  };

  return { container, scrollToCalls };
}

describe("cycleVerticalScrollForward", () => {
  it("scrolls down one viewport chunk when content remains below", () => {
    const { container, scrollToCalls } = mockScrollContainer({
      scrollTop: 0,
      scrollHeight: 800,
      clientHeight: 400,
    });

    cycleVerticalScrollForward(container);

    expect(scrollToCalls).toEqual([{ top: 300, behavior: "smooth" }]);
  });

  it("scrolls to the bottom instead of wrapping when the next step would overshoot", () => {
    const { container, scrollToCalls } = mockScrollContainer({
      scrollTop: 0,
      scrollHeight: 500,
      clientHeight: 400,
    });

    cycleVerticalScrollForward(container);

    expect(scrollToCalls).toEqual([{ top: 100, behavior: "smooth" }]);
    expect(scrollToCalls[0]?.top).not.toBe(0);
  });

  it("clamps to max scroll when near the bottom", () => {
    const { container, scrollToCalls } = mockScrollContainer({
      scrollTop: 80,
      scrollHeight: 500,
      clientHeight: 400,
    });

    cycleVerticalScrollForward(container);

    expect(scrollToCalls).toEqual([{ top: 100, behavior: "smooth" }]);
  });
});

describe("cycleVerticalScrollBackward", () => {
  it("scrolls up one viewport chunk when content remains above", () => {
    const { container, scrollToCalls } = mockScrollContainer({
      scrollTop: 300,
      scrollHeight: 800,
      clientHeight: 400,
    });

    cycleVerticalScrollBackward(container);

    expect(scrollToCalls).toEqual([{ top: 0, behavior: "smooth" }]);
  });

  it("clamps to the top when the previous step would overshoot", () => {
    const { container, scrollToCalls } = mockScrollContainer({
      scrollTop: 50,
      scrollHeight: 800,
      clientHeight: 400,
    });

    cycleVerticalScrollBackward(container);

    expect(scrollToCalls).toEqual([{ top: 0, behavior: "smooth" }]);
  });
});
