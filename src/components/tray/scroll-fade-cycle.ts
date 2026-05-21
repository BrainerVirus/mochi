const TRAY_SEGMENT_SELECTOR = '[data-slot="toggle-group-item"]';

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

export function cycleVerticalScroll(container: HTMLDivElement) {
  const step = container.clientHeight * 0.75;
  const maxScroll = container.scrollHeight - container.clientHeight;
  const next = container.scrollTop + step;

  if (next >= maxScroll - 1) {
    container.scrollTo({ top: 0, behavior: "smooth" });
    return;
  }

  container.scrollTo({ top: next, behavior: "smooth" });
}
