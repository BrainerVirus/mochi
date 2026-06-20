// @vitest-environment happy-dom
import { cleanup, fireEvent, render } from "@testing-library/react";
import gsap from "gsap";
import { createElement } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ScrollFadeEdgeOverlays } from "./scroll-fade-overlays";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("ScrollFadeEdgeOverlays vertical chevrons", () => {
  it("uses accessible CSS visibility states for both vertical sides", () => {
    const props = {
      isHorizontal: false,
      onCycleBackward: vi.fn<() => void>(),
      onCycleForward: vi.fn<() => void>(),
    };
    const { getByLabelText, rerender } = render(
      createElement(ScrollFadeEdgeOverlays, {
        ...props,
        canScrollStart: false,
        canScrollEnd: false,
      }),
    );
    const start = getByLabelText("Scroll up for more");
    const end = getByLabelText("Scroll down for more");
    if (!(start instanceof HTMLButtonElement) || !(end instanceof HTMLButtonElement)) {
      throw new TypeError("Expected vertical chevron buttons");
    }

    for (const [button, translation] of [
      [start, "-translate-y-1"],
      [end, "translate-y-1"],
    ] as const) {
      expect(button.getAttribute("aria-hidden")).toBe("true");
      expect(button.tabIndex).toBe(-1);
      expect(button.className).toContain("pointer-events-none");
      expect(button.className).toContain("opacity-0");
      expect(button.className).toContain(translation);
      expect(button.className).toContain("transition-[opacity,translate]");
      expect(button.className).toContain("duration-200");
      expect(button.className).toContain("ease-out");
      expect(button.className).toContain("motion-reduce:transition-none");
      expect(button.disabled).toBe(true);
    }

    rerender(
      createElement(ScrollFadeEdgeOverlays, {
        ...props,
        canScrollStart: true,
        canScrollEnd: true,
      }),
    );

    for (const button of [start, end]) {
      expect(button.getAttribute("aria-hidden")).toBe("false");
      expect(button.tabIndex).toBe(0);
      expect(button.className).toContain("opacity-100");
      expect(button.disabled).toBe(false);
    }
  });
});

describe("ScrollFadeEdgeOverlays vertical interactions", () => {
  it("releases focus and blocks activation when a visible chevron becomes hidden", () => {
    const onCycleBackward = vi.fn<() => void>();
    const props = {
      isHorizontal: false,
      onCycleBackward,
      onCycleForward: vi.fn<() => void>(),
    };
    const { getByRole, rerender } = render(
      createElement(ScrollFadeEdgeOverlays, {
        ...props,
        canScrollStart: true,
        canScrollEnd: true,
      }),
    );
    const button = getByRole("button", { name: "Scroll up for more" });

    button.focus();
    expect(document.activeElement).toBe(button);

    rerender(
      createElement(ScrollFadeEdgeOverlays, {
        ...props,
        canScrollStart: false,
        canScrollEnd: true,
      }),
    );

    expect(document.activeElement).not.toBe(button);
    fireEvent.keyDown(button, { key: "Enter" });
    fireEvent.keyUp(button, { key: "Enter" });
    fireEvent.click(button);
    expect(onCycleBackward).not.toHaveBeenCalled();
  });

  it("preserves the visible vertical chevron callbacks", () => {
    const onCycleBackward = vi.fn<() => void>();
    const onCycleForward = vi.fn<() => void>();
    const { getByRole } = render(
      createElement(ScrollFadeEdgeOverlays, {
        isHorizontal: false,
        canScrollStart: true,
        canScrollEnd: true,
        onCycleBackward,
        onCycleForward,
      }),
    );

    fireEvent.click(getByRole("button", { name: "Scroll up for more" }));
    fireEvent.click(getByRole("button", { name: "Scroll down for more" }));

    expect(onCycleBackward).toHaveBeenCalledOnce();
    expect(onCycleForward).toHaveBeenCalledOnce();
  });
});

describe("ScrollFadeEdgeOverlays vertical animation", () => {
  it("rapidly toggles visibility without reconstructing GSAP media contexts", () => {
    const matchMediaSpy = vi.spyOn(gsap, "matchMedia");
    const props = {
      isHorizontal: false,
      onCycleBackward: vi.fn<() => void>(),
      onCycleForward: vi.fn<() => void>(),
    };
    const { rerender } = render(
      createElement(ScrollFadeEdgeOverlays, {
        ...props,
        canScrollStart: false,
        canScrollEnd: false,
      }),
    );

    for (let index = 0; index < 12; index += 1) {
      rerender(
        createElement(ScrollFadeEdgeOverlays, {
          ...props,
          canScrollStart: index % 2 === 0,
          canScrollEnd: index % 2 !== 0,
        }),
      );
    }

    expect(matchMediaSpy).not.toHaveBeenCalled();
  });
});
