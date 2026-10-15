import { useEffect, useRef } from "react";
import { LAYOUT } from "../utils/constants";

const SHOW_AT = 25; // px pulled before the indicator appears
const TRIGGER_AT = LAYOUT.pullToRefreshTriggerPx; // px pulled to refresh on release
const HOLD_AT = 56; // px the content stays down while refreshing
const MAX_PULL = 128;

// Resistance: tracks the finger early on, then levels off at MAX_PULL.
// TRIGGER_AT is reached after roughly 200px of finger travel.
function resist(distance: number): number {
  return MAX_PULL * (1 - Math.exp(-distance / 120));
}

/**
 * Pull-down-to-refresh on a scroll container, starting only when it's
 * scrolled to the top. Moves the content and flags the indicator directly via
 * the DOM rather than React state, so pulling doesn't re-render the list.
 * Disabled when `onRefresh` is undefined.
 */
export function usePullToRefresh(
  scrollRef: React.RefObject<HTMLElement | null>,
  contentRef: React.RefObject<HTMLElement | null>,
  indicatorRef: React.RefObject<HTMLElement | null>,
  onRefresh: (() => void) | undefined,
  refreshing: boolean,
) {
  // The listeners are attached once; this keeps them calling the latest callback.
  const onRefreshRef = useRef(onRefresh);
  useEffect(() => {
    onRefreshRef.current = onRefresh;
  });
  const enabled = onRefresh !== undefined;

  useEffect(() => {
    const scroll = scrollRef.current;
    const content = contentRef.current;
    const indicator = indicatorRef.current;
    if (!enabled || !scroll || !content || !indicator) return;

    let startY = 0;
    let pulled = 0;

    function move(offset: number, animate: boolean) {
      content!.style.transition = animate ? "transform 0.2s ease" : "";
      content!.style.transform = offset ? `translateY(${offset}px)` : "";
    }

    function onMove(e: TouchEvent) {
      const distance = e.touches[0].clientY - startY;
      if (distance <= 0) {
        pulled = 0;
        indicator!.dataset.state = "";
        return move(0, false);
      }
      // Stop the list (and iOS rubber-banding) from moving while pulling.
      e.preventDefault();
      pulled = resist(distance);
      indicator!.dataset.state =
        pulled >= TRIGGER_AT ? "ready" : pulled >= SHOW_AT ? "pulling" : "";
      move(pulled, false);
    }

    function onEnd() {
      const triggered = pulled >= TRIGGER_AT;
      pulled = 0;
      indicator!.dataset.state = "";
      // When triggered, the `refreshing` effect below takes over the offset.
      move(triggered ? HOLD_AT : 0, true);
      if (triggered) onRefreshRef.current?.();
      scroll!.removeEventListener("touchmove", onMove);
      scroll!.removeEventListener("touchend", onEnd);
      scroll!.removeEventListener("touchcancel", onEnd);
    }

    function onStart(e: TouchEvent) {
      // iOS rubber-banding can push scrollTop slightly below 0 at the top.
      if (scroll!.scrollTop > 0) return;
      startY = e.touches[0].clientY;
      // Not passive: onMove needs preventDefault.
      scroll!.addEventListener("touchmove", onMove, { passive: false });
      scroll!.addEventListener("touchend", onEnd);
      scroll!.addEventListener("touchcancel", onEnd);
    }

    scroll.addEventListener("touchstart", onStart, { passive: true });
    return () => {
      scroll.removeEventListener("touchstart", onStart);
      scroll.removeEventListener("touchmove", onMove);
      scroll.removeEventListener("touchend", onEnd);
      scroll.removeEventListener("touchcancel", onEnd);
    };
  }, [scrollRef, contentRef, indicatorRef, enabled]);

  // Hold the content down while refreshing so the spinner has room, and slide
  // it back once done.
  useEffect(() => {
    const content = contentRef.current;
    if (!enabled || !content) return;
    content.style.transition = "transform 0.2s ease";
    content.style.transform = refreshing ? `translateY(${HOLD_AT}px)` : "";
  }, [contentRef, enabled, refreshing]);
}
