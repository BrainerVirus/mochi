/** Clears DOM focus for interactive elements inside the tray panel root. */
export function clearTrayPanelFocus(root: ParentNode = document): void {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement) || active === document.body) {
    return;
  }

  if (!root.contains(active)) {
    return;
  }

  active.blur();
}

/** Blurs on the next frame — macOS webviews may restore focus asynchronously. */
export function clearTrayPanelFocusDeferred(root: ParentNode = document): void {
  clearTrayPanelFocus(root);
  requestAnimationFrame(() => {
    clearTrayPanelFocus(root);
  });
}
