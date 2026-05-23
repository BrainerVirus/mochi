import { describe, expect, it } from "vitest";

import { measureScrollOverflow } from "@/components/tray/use-scroll-overflow";

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
