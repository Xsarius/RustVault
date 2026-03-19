/**
 * SwipeableRow — horizontal swipe gesture container for list items.
 *
 * Reveals configurable action buttons when swiped left.
 * Used on transaction rows for quick categorize / delete actions.
 */

import {
  type ParentComponent,
  type JSX,
  createSignal,
  onMount,
  onCleanup,
} from "solid-js";

interface SwipeAction {
  label: string;
  icon?: JSX.Element;
  color: string; // Tailwind bg class, e.g. "bg-red-500"
  onAction: () => void;
}

interface SwipeableRowProps {
  /** Actions revealed when swiping left (max 3 recommended) */
  actions: SwipeAction[];
  /** Width of each action button in px (default: 72) */
  actionWidth?: number;
}

export const SwipeableRow: ParentComponent<SwipeableRowProps> = (props) => {
  const actionWidth = props.actionWidth ?? 72;
  const totalReveal = props.actions.length * actionWidth;

  let rowRef!: HTMLDivElement;
  const [offset, setOffset] = createSignal(0);
  const [snapped, setSnapped] = createSignal(false);

  let startX = 0;
  let startOffset = 0;
  let tracking = false;

  function onTouchStart(e: TouchEvent) {
    startX = e.touches[0].clientX;
    startOffset = offset();
    tracking = true;
  }

  function onTouchMove(e: TouchEvent) {
    if (!tracking) return;
    const delta = e.touches[0].clientX - startX + startOffset;
    // Only allow swiping left (negative offset) up to the total reveal width.
    const clamped = Math.max(-totalReveal, Math.min(0, delta));
    setOffset(clamped);
  }

  function onTouchEnd() {
    if (!tracking) return;
    tracking = false;

    // Snap open or closed based on threshold.
    if (offset() < -(totalReveal / 2)) {
      setOffset(-totalReveal);
      setSnapped(true);
    } else {
      setOffset(0);
      setSnapped(false);
    }
  }

  function close() {
    setOffset(0);
    setSnapped(false);
  }

  onMount(() => {
    rowRef.addEventListener("touchstart", onTouchStart, { passive: true });
    rowRef.addEventListener("touchmove", onTouchMove, { passive: true });
    rowRef.addEventListener("touchend", onTouchEnd, { passive: true });
  });

  onCleanup(() => {
    rowRef.removeEventListener("touchstart", onTouchStart);
    rowRef.removeEventListener("touchmove", onTouchMove);
    rowRef.removeEventListener("touchend", onTouchEnd);
  });

  return (
    <div
      ref={rowRef}
      class="relative overflow-hidden"
      onClick={() => {
        if (snapped()) close();
      }}
    >
      {/* Action buttons (revealed on swipe) */}
      <div
        class="absolute inset-y-0 right-0 flex"
        style={{ width: `${totalReveal}px` }}
      >
        {props.actions.map((action) => (
          <button
            class={`flex flex-col items-center justify-center gap-1 ${action.color} text-white text-xs font-medium cursor-pointer`}
            style={{ width: `${actionWidth}px` }}
            onClick={(e) => {
              e.stopPropagation();
              action.onAction();
              close();
            }}
          >
            {action.icon}
            {action.label}
          </button>
        ))}
      </div>

      {/* Row content */}
      <div
        style={{
          transform: `translateX(${offset()}px)`,
          transition: tracking ? "none" : "transform 0.2s ease",
        }}
      >
        {props.children}
      </div>
    </div>
  );
};
