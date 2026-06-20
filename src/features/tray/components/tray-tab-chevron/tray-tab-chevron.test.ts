// @vitest-environment happy-dom
import { cleanup, fireEvent, render } from "@testing-library/react";
import gsap from "gsap";
import { createElement } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { getTrayTabChevronButtonClassName } from "@/features/tray/components/tray-tab-chevron-class-name";

import { TrayTabChevron } from "./tray-tab-chevron";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("getTrayTabChevronButtonClassName", () => {
  it("keeps overflow chevrons ghost-style and visually legible", () => {
    const className = getTrayTabChevronButtonClassName(true);

    expect(className).toContain("bg-transparent");
    expect(className).toContain("text-foreground");
    expect(className).toContain("hover:bg-transparent");
    expect(className).not.toContain("bg-background/85");
    expect(className).not.toContain("ring-1");
    expect(className).not.toContain("shadow-sm");
    expect(className).not.toContain("backdrop-blur");
  });

  it("disables pointer events while hidden", () => {
    expect(getTrayTabChevronButtonClassName(false)).toContain("pointer-events-none");
  });
});

describe("TrayTabChevron", () => {
  it.each([
    ["start", "-translate-x-1"],
    ["end", "translate-x-1"],
  ] as const)("uses CSS visibility states on the %s side", (side, hiddenTranslation) => {
    const { container, rerender } = render(
      createElement(TrayTabChevron, { side, visible: false, onCycle: vi.fn<() => void>() }),
    );
    const column = container.firstElementChild;
    if (!(column instanceof HTMLElement)) {
      throw new TypeError("Expected a chevron column");
    }
    const button = column.querySelector("button");
    if (!(button instanceof HTMLButtonElement)) {
      throw new TypeError("Expected a chevron button");
    }

    expect(column.getAttribute("aria-hidden")).toBe("true");
    expect(column.className).toContain("opacity-0");
    expect(column.className).toContain(hiddenTranslation);
    expect(column.className).toContain("transition-[opacity,translate]");
    expect(column.className).toContain("duration-200");
    expect(column.className).toContain("ease-out");
    expect(column.className).toContain("motion-reduce:transition-none");
    expect(button.tabIndex).toBe(-1);
    expect(button.disabled).toBe(true);
    expect(button.className).toContain("pointer-events-none");

    rerender(createElement(TrayTabChevron, { side, visible: true, onCycle: vi.fn<() => void>() }));

    expect(column.getAttribute("aria-hidden")).toBe("false");
    expect(column.className).toContain("opacity-100");
    expect(column.className).not.toContain(hiddenTranslation);
    expect(button.tabIndex).toBe(0);
    expect(button.disabled).toBe(false);
  });

  it("releases focus and blocks activation when a visible chevron becomes hidden", () => {
    const onCycle = vi.fn<() => void>();
    const props = { side: "end" as const, onCycle };
    const { getByRole, rerender } = render(
      createElement(TrayTabChevron, { ...props, visible: true }),
    );
    const button = getByRole("button", { name: "Show more tabs" });

    button.focus();
    expect(document.activeElement).toBe(button);

    rerender(createElement(TrayTabChevron, { ...props, visible: false }));

    expect(document.activeElement).not.toBe(button);
    fireEvent.keyDown(button, { key: "Enter" });
    fireEvent.keyUp(button, { key: "Enter" });
    fireEvent.click(button);
    expect(onCycle).not.toHaveBeenCalled();
  });

  it("keeps the visible chevron callback behavior", () => {
    const onCycle = vi.fn<() => void>();
    const { getByRole } = render(
      createElement(TrayTabChevron, { side: "end", visible: true, onCycle }),
    );

    fireEvent.click(getByRole("button", { name: "Show more tabs" }));

    expect(onCycle).toHaveBeenCalledOnce();
  });

  it("rapidly toggles visibility without reconstructing GSAP media contexts", () => {
    const matchMediaSpy = vi.spyOn(gsap, "matchMedia");
    const props = { side: "end" as const, onCycle: vi.fn<() => void>() };
    const { rerender } = render(createElement(TrayTabChevron, { ...props, visible: false }));

    for (let index = 0; index < 12; index += 1) {
      rerender(createElement(TrayTabChevron, { ...props, visible: index % 2 === 0 }));
    }

    expect(matchMediaSpy).not.toHaveBeenCalled();
  });
});
