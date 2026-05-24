export interface TraySegmentHoverState {
  pointerId: string | null;
  focusedId: string | null;
}

export function createTraySegmentHoverState(): TraySegmentHoverState {
  return {
    pointerId: null,
    focusedId: null,
  };
}

export function setTraySegmentPointerHover(state: TraySegmentHoverState, tabId: string) {
  state.pointerId = tabId;
}

export function clearTraySegmentPointerHover(state: TraySegmentHoverState, tabId: string) {
  if (state.pointerId === tabId) {
    state.pointerId = null;
  }
}

export function clearTraySegmentHover(state: TraySegmentHoverState) {
  state.pointerId = null;
  state.focusedId = null;
}

export function getTraySegmentHoverTarget(state: TraySegmentHoverState) {
  return state.pointerId;
}
