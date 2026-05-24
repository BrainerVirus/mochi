export interface SegmentTrackResizeObserver {
  disconnect: () => void;
}

/** Debounce track resize to one rAF per frame — avoids layout feedback during GSAP width tweens. */
export function observeSegmentTrackResize(
  track: HTMLElement,
  onResize: () => void,
): SegmentTrackResizeObserver {
  let frameId = 0;

  const scheduleResize = () => {
    cancelAnimationFrame(frameId);
    frameId = requestAnimationFrame(onResize);
  };

  const resizeObserver = new ResizeObserver(scheduleResize);
  resizeObserver.observe(track);

  return {
    disconnect() {
      cancelAnimationFrame(frameId);
      resizeObserver.disconnect();
    },
  };
}
