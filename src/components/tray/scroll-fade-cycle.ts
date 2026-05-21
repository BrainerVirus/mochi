const TRAY_SEGMENT_SELECTOR = '[data-slot="toggle-group-item"]';

type ScrollableElement = {
  scrollTop: number;
  scrollHeight: number;
  clientHeight: number;
  scrollTo(options: ScrollToOptions): void;
};

function getTriggerScrollLeft(container: HTMLDivElement, trigger: HTMLElement): number {
  const containerRect = container.getBoundingClientRect();
  const triggerRect = trigger.getBoundingClientRect();
  return triggerRect.left - containerRect.left + container.scrollLeft;
}

export function cycleHorizontalScrollForward(container: HTMLDivElement, fadeInset: number) {
  const triggers = [...container.querySelectorAll<HTMLElement>(TRAY_SEGMENT_SELECTOR)];
  if (triggers.length === 0) {
    return;
  }

  const visibleRight = container.scrollLeft + container.clientWidth - fadeInset;

  for (const trigger of triggers) {
    const left = getTriggerScrollLeft(container, trigger);
    const right = left + trigger.offsetWidth;

    if (right > visibleRight + 1) {
      container.scrollTo({ left, behavior: "smooth" });
      return;
    }
  }

  const maxScroll = Math.max(0, container.scrollWidth - container.clientWidth);
  container.scrollTo({ left: maxScroll, behavior: "smooth" });
}

export function cycleHorizontalScrollBackward(container: HTMLDivElement, fadeInset: number) {
  const triggers = [...container.querySelectorAll<HTMLElement>(TRAY_SEGMENT_SELECTOR)];
  if (triggers.length === 0) {
    return;
  }

  const visibleLeft = container.scrollLeft + fadeInset;

  for (let index = triggers.length - 1; index >= 0; index -= 1) {
    const trigger = triggers[index];
    const left = getTriggerScrollLeft(container, trigger);

    if (left < visibleLeft - 1) {
      container.scrollTo({ left: Math.max(0, left - fadeInset), behavior: "smooth" });
      return;
    }
  }

  container.scrollTo({ left: 0, behavior: "smooth" });
}

function verticalScrollChunk(container: ScrollableElement): number {
  return Math.max(container.clientHeight, 1) * 0.75;
}

export function cycleVerticalScrollForward(container: ScrollableElement) {
  const maxScroll = Math.max(0, container.scrollHeight - container.clientHeight);
  const next = Math.min(container.scrollTop + verticalScrollChunk(container), maxScroll);

  container.scrollTo({ top: next, behavior: "smooth" });
}

export function cycleVerticalScrollBackward(container: ScrollableElement) {
  const next = Math.max(0, container.scrollTop - verticalScrollChunk(container));

  container.scrollTo({ top: next, behavior: "smooth" });
}
