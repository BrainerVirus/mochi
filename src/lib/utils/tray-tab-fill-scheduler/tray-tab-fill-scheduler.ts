const readyQueue = new Set<() => void>();

let tabFillPending = false;
let tabFillPendingToken = 0;

export function markTrayTabFillPending(): number {
  tabFillPending = true;
  tabFillPendingToken += 1;

  return tabFillPendingToken;
}

export function getTrayTabFillPendingToken(): number {
  return tabFillPendingToken;
}

export function markTrayTabFillReady(token = tabFillPendingToken): void {
  if (token !== tabFillPendingToken) {
    return;
  }

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
  tabFillPendingToken = 0;
  readyQueue.clear();
}
