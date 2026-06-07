import { describe, expect, it } from "vitest";

import {
  createTraySegmentIndicatorMachine,
  transitionTraySegmentIndicator,
  type TraySegmentIndicatorState,
} from "@/features/tray/components/tray-segment-indicator-machine";

describe("tray segment indicator machine", () => {
  it("places the hover indicator on the first item entered from outside the rail", () => {
    const result = transitionTraySegmentIndicator(createTraySegmentIndicatorMachine(), {
      type: "ITEM_ENTER",
      tabId: "codex",
    });

    expect(result.state).toEqual({ status: "hovering", hoveredId: "codex" });
    expect(result.commands).toEqual([{ type: "placeHover", tabId: "codex" }]);
  });

  it("moves the hover indicator when crossing directly between tabs inside the rail", () => {
    const state: TraySegmentIndicatorState = { status: "hovering", hoveredId: "overview" };

    const result = transitionTraySegmentIndicator(state, {
      type: "ITEM_ENTER",
      tabId: "codex",
    });

    expect(result.state).toEqual({ status: "hovering", hoveredId: "codex" });
    expect(result.commands).toEqual([{ type: "moveHover", tabId: "codex" }]);
  });

  it("resets hover origin after leaving the rail", () => {
    const leave = transitionTraySegmentIndicator(
      { status: "hovering", hoveredId: "codex" },
      { type: "RAIL_LEAVE" },
    );
    const reenter = transitionTraySegmentIndicator(leave.state, {
      type: "ITEM_ENTER",
      tabId: "cursor",
    });

    expect(leave.state).toEqual({ status: "outside" });
    expect(leave.commands).toEqual([{ type: "hideHover", immediate: false }]);
    expect(reenter.commands).toEqual([{ type: "placeHover", tabId: "cursor" }]);
  });

  it("hides hover immediately and moves active when selecting the hovered tab", () => {
    const result = transitionTraySegmentIndicator(
      { status: "hovering", hoveredId: "codex" },
      { type: "SELECT", tabId: "codex" },
    );

    expect(result.state).toEqual({ status: "selecting", selectedId: "codex" });
    expect(result.commands).toEqual([
      { type: "hideHover", immediate: true },
      { type: "moveActive", tabId: "codex" },
    ]);
  });

  it("hides hover immediately and moves active without hover handoff when selecting another tab", () => {
    const result = transitionTraySegmentIndicator(
      { status: "hovering", hoveredId: "codex" },
      { type: "SELECT", tabId: "cursor" },
    );

    expect(result.state).toEqual({ status: "selecting", selectedId: "cursor" });
    expect(result.commands).toEqual([
      { type: "hideHover", immediate: true },
      { type: "moveActive", tabId: "cursor" },
    ]);
    expect(result.commands).not.toContainEqual(expect.objectContaining({ type: "moveHover" }));
    expect(result.commands).not.toContainEqual(expect.objectContaining({ type: "placeHover" }));
  });
});
