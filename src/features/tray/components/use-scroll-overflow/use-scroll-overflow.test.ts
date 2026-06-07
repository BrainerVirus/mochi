import { describe, expect, it } from "vitest";

import {
  measureScrollOverflow,
  measureScrollOverflowWithHysteresis,
  SCROLL_OVERFLOW_HIDE_THRESHOLD,
  SCROLL_OVERFLOW_SHOW_THRESHOLD,
} from "@/features/tray/components/use-scroll-overflow";

describe("measureScrollOverflow", () => {
  it("detects vertical overflow from scroll metrics", () => {
    expect(
      measureScrollOverflow(
        {
          scrollTop: 0,
          clientHeight: 100,
          scrollHeight: 240,
          scrollLeft: 0,
          clientWidth: 200,
          scrollWidth: 200,
        },
        "vertical",
      ),
    ).toEqual({
      canScrollStart: false,
      canScrollEnd: true,
    });
  });

  it("detects horizontal overflow from scroll metrics", () => {
    expect(
      measureScrollOverflow(
        {
          scrollTop: 0,
          clientHeight: 44,
          scrollHeight: 44,
          scrollLeft: 0,
          clientWidth: 200,
          scrollWidth: 320,
        },
        "horizontal",
      ),
    ).toEqual({
      canScrollStart: false,
      canScrollEnd: true,
    });
  });
});

describe("measureScrollOverflowWithHysteresis", () => {
  it("shows start overflow after the show threshold", () => {
    expect(
      measureScrollOverflowWithHysteresis(
        {
          scrollTop: SCROLL_OVERFLOW_SHOW_THRESHOLD + 1,
          clientHeight: 100,
          scrollHeight: 240,
          scrollLeft: 0,
          clientWidth: 200,
          scrollWidth: 200,
        },
        "vertical",
        { canScrollStart: false, canScrollEnd: false },
      ),
    ).toEqual({
      canScrollStart: true,
      canScrollEnd: true,
    });
  });

  it("keeps start overflow visible until the hide threshold", () => {
    expect(
      measureScrollOverflowWithHysteresis(
        {
          scrollTop: SCROLL_OVERFLOW_HIDE_THRESHOLD,
          clientHeight: 100,
          scrollHeight: 240,
          scrollLeft: 0,
          clientWidth: 200,
          scrollWidth: 200,
        },
        "vertical",
        { canScrollStart: true, canScrollEnd: true },
      ),
    ).toEqual({
      canScrollStart: false,
      canScrollEnd: true,
    });
  });

  it("applies horizontal hysteresis at the trailing edge", () => {
    const metrics = {
      scrollTop: 0,
      clientHeight: 44,
      scrollHeight: 44,
      scrollLeft: 119,
      clientWidth: 200,
      scrollWidth: 320,
    };

    expect(
      measureScrollOverflowWithHysteresis(metrics, "horizontal", {
        canScrollStart: true,
        canScrollEnd: true,
      }),
    ).toEqual({
      canScrollStart: true,
      canScrollEnd: false,
    });

    expect(
      measureScrollOverflowWithHysteresis({ ...metrics, scrollLeft: 115 }, "horizontal", {
        canScrollStart: true,
        canScrollEnd: true,
      }),
    ).toEqual({
      canScrollStart: true,
      canScrollEnd: true,
    });
  });
});
