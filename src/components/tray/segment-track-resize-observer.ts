export interface SegmentTrackResizeObserver {
  disconnect: () => void;
}

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
