export type TraySegmentIndicatorState =
  | { status: "outside" }
  | { status: "hovering"; hoveredId: string }
  | { status: "selecting"; selectedId: string };

export type TraySegmentIndicatorEvent =
  | { type: "ITEM_ENTER"; tabId: string }
  | { type: "RAIL_LEAVE" }
  | { type: "SELECT"; tabId: string };

export type TraySegmentIndicatorCommand =
  | { type: "placeHover"; tabId: string }
  | { type: "moveHover"; tabId: string }
  | { type: "hideHover"; immediate: boolean }
  | { type: "moveActive"; tabId: string }
  | { type: "snapActive"; tabId: string };

export interface TraySegmentIndicatorTransition {
  state: TraySegmentIndicatorState;
  commands: TraySegmentIndicatorCommand[];
}

export function createTraySegmentIndicatorMachine(): TraySegmentIndicatorState {
  return { status: "outside" };
}

export function transitionTraySegmentIndicator(
  state: TraySegmentIndicatorState,
  event: TraySegmentIndicatorEvent,
): TraySegmentIndicatorTransition {
  if (event.type === "ITEM_ENTER") {
    if (state.status === "hovering") {
      return {
        state: { status: "hovering", hoveredId: event.tabId },
        commands:
          state.hoveredId === event.tabId ? [] : [{ type: "moveHover", tabId: event.tabId }],
      };
    }

    return {
      state: { status: "hovering", hoveredId: event.tabId },
      commands: [{ type: "placeHover", tabId: event.tabId }],
    };
  }

  if (event.type === "RAIL_LEAVE") {
    return {
      state: { status: "outside" },
      commands: state.status === "hovering" ? [{ type: "hideHover", immediate: false }] : [],
    };
  }

  return {
    state: { status: "selecting", selectedId: event.tabId },
    commands: [
      ...(state.status === "hovering" ? [{ type: "hideHover" as const, immediate: true }] : []),
      { type: "moveActive", tabId: event.tabId },
    ],
  };
}
