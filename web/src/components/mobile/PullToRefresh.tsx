/**
 * PullToRefresh — native-feeling pull-to-refresh wrapper.
 *
 * Detects a downward touch swipe at the top of a scrollable container
 * and calls the `onRefresh` callback when the threshold is met.
 * Shows a spinner while `refreshing()` is true.
 *
 * Works on both native Capacitor and browser environments.
 */

import {
  type ParentComponent,
  createSignal,
  onMount,
  onCleanup,
  Show,
} from "solid-js";

interface PullToRefreshProps {
  onRefresh: () => void | Promise<void>;
  refreshing: boolean;
  /** Pull distance in px before triggering refresh (default: 72) */
  threshold?: number;
}

export const PullToRefresh: ParentComponent<PullToRefreshProps> = (props) => {
  const threshold = props.threshold ?? 72;

  let containerRef!: HTMLDivElement;
  const [pullDistance, setPullDistance] = createSignal(0);
  const [triggered, setTriggered] = createSignal(false);

  let startY = 0;
  let tracking = false;

  function onTouchStart(e: TouchEvent) {
    // Only start tracking when scrolled to the top.
    if (containerRef.scrollTop > 0) return;
    startY = e.touches[0].clientY;
    tracking = true;
    setTriggered(false);
  }

  function onTouchMove(e: TouchEvent) {
    if (!tracking || props.refreshing) return;
    const delta = e.touches[0].clientY - startY;
    if (delta <= 0) {
      setPullDistance(0);
      return;
    }
    // Apply rubberbanding: slow down at larger distances.
    const rubberband = Math.min(delta * 0.5, threshold * 1.5);
    setPullDistance(rubberband);

    if (rubberband >= threshold && !triggered()) {
      setTriggered(true);
    }

    // Prevent default scroll when we're handling the pull gesture.
    if (delta > 0) e.preventDefault();
  }

  async function onTouchEnd() {
    if (!tracking) return;
    tracking = false;

    if (triggered() && !props.refreshing) {
      await props.onRefresh();
    }

    setPullDistance(0);
    setTriggered(false);
  }

  onMount(() => {
    containerRef.addEventListener("touchstart", onTouchStart, { passive: true });
    containerRef.addEventListener("touchmove", onTouchMove, { passive: false });
    containerRef.addEventListener("touchend", onTouchEnd, { passive: true });
  });

  onCleanup(() => {
    containerRef.removeEventListener("touchstart", onTouchStart);
    containerRef.removeEventListener("touchmove", onTouchMove);
    containerRef.removeEventListener("touchend", onTouchEnd);
  });

  return (
    <div ref={containerRef} class="relative overflow-y-auto h-full">
      {/* Pull indicator */}
      <Show when={pullDistance() > 0 || props.refreshing}>
        <div
          class="absolute inset-x-0 top-0 flex items-center justify-center z-10 pointer-events-none transition-[height] duration-150"
          style={{ height: `${props.refreshing ? 48 : pullDistance()}px` }}
        >
          <div
            class={`h-8 w-8 rounded-full border-2 border-primary border-t-transparent ${
              props.refreshing ? "animate-spin" : ""
            }`}
            style={{
              opacity: props.refreshing
                ? "1"
                : `${Math.min(pullDistance() / threshold, 1)}`,
              transform: `rotate(${props.refreshing ? "" : `${(pullDistance() / threshold) * 180}deg`})`,
            }}
          />
        </div>
      </Show>

      {/* Content */}
      <div
        style={{
          transform: `translateY(${props.refreshing ? 48 : pullDistance()}px)`,
          transition: !tracking ? "transform 0.2s ease" : "none",
        }}
      >
        {props.children}
      </div>
    </div>
  );
};
