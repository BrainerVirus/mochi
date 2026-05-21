const readyQueue = new Set<() => void>();

let tabFillPending = false;

export function markTrayTabFillPending(): void {
  tabFillPending = true;
}

export function markTrayTabFillReady(): void {
  tabFillPending = false;

  for (const run of readyQueue) {
    run();
  }

  readyQueue.clear();
}

export function isTrayTabFillPending(): boolean {
  return tabFillPending;
}

export function runWhenTrayTabFillReady(run: () => void): () => void {
  if (!tabFillPending) {
    run();
    return () => {};
  }

  readyQueue.add(run);

  return () => {
    readyQueue.delete(run);
  };
}

export function resetTrayTabFillSchedulerForTests(): void {
  tabFillPending = false;
  readyQueue.clear();
}
