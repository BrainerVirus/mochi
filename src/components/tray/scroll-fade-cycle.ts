export function cycleHorizontalScrollForward(container: HTMLDivElement, fadeInset: number) {
  const triggers = [...container.querySelectorAll<HTMLElement>('[data-slot="tabs-trigger"]')];
  const visibleRight = container.scrollLeft + container.clientWidth - fadeInset;

  for (const trigger of triggers) {
    if (trigger.offsetLeft + trigger.offsetWidth > visibleRight + 1) {
      container.scrollTo({ left: trigger.offsetLeft, behavior: "smooth" });
      return;
    }
  }

  container.scrollTo({ left: 0, behavior: "smooth" });
}

export function cycleHorizontalScrollBackward(container: HTMLDivElement, fadeInset: number) {
  const triggers = [...container.querySelectorAll<HTMLElement>('[data-slot="tabs-trigger"]')];
  const visibleLeft = container.scrollLeft + fadeInset;

  for (let index = triggers.length - 1; index >= 0; index -= 1) {
    const trigger = triggers[index];
    if (trigger.offsetLeft < visibleLeft - 1) {
      container.scrollTo({ left: trigger.offsetLeft, behavior: "smooth" });
      return;
    }
  }

  const maxScroll = container.scrollWidth - container.clientWidth;
  container.scrollTo({ left: maxScroll, behavior: "smooth" });
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
