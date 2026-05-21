export function nextTabFillActivationState(
  previous: { tab: string; generation: number },
  nextTab: string,
): { tab: string; generation: number } {
  return {
    tab: nextTab,
    generation: previous.generation + 1,
  };
}

export function formatTabFillActivationKey(state: { tab: string; generation: number }): string {
  return `${state.tab}:${state.generation}`;
}
