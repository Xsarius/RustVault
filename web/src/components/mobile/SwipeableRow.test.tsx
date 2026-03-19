import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@solidjs/testing-library";
import { SwipeableRow } from "~/components/mobile/SwipeableRow";

function fireSwipe(element: HTMLElement, deltaX: number) {
  const startX = 200;
  const endX = startX + deltaX;
  element.dispatchEvent(
    new TouchEvent("touchstart", {
      bubbles: true,
      touches: [new Touch({ identifier: 1, target: element, clientX: startX, clientY: 0 })],
    }),
  );
  element.dispatchEvent(
    new TouchEvent("touchmove", {
      bubbles: true,
      touches: [new Touch({ identifier: 1, target: element, clientX: endX, clientY: 0 })],
    }),
  );
  element.dispatchEvent(
    new TouchEvent("touchend", {
      bubbles: true,
      changedTouches: [new Touch({ identifier: 1, target: element, clientX: endX, clientY: 0 })],
    }),
  );
}

describe("SwipeableRow", () => {
  const actions = [
    { label: "Delete", color: "bg-red-500", onAction: vi.fn() },
  ];

  it("renders its children", () => {
    render(() => (
      <SwipeableRow actions={actions}>
        <span>row content</span>
      </SwipeableRow>
    ));
    expect(screen.getByText("row content")).toBeInTheDocument();
  });

  it("renders action button labels", () => {
    render(() => (
      <SwipeableRow actions={actions}>
        <span>row</span>
      </SwipeableRow>
    ));
    expect(screen.getByText("Delete")).toBeInTheDocument();
  });

  it("calls onAction when action button is clicked", async () => {
    const onAction = vi.fn();
    render(() => (
      <SwipeableRow actions={[{ label: "Remove", color: "bg-red-500", onAction }]}>
        <span>row</span>
      </SwipeableRow>
    ));
    // Click the action button directly (simulates tapping after reveal).
    screen.getByText("Remove").click();
    expect(onAction).toHaveBeenCalledOnce();
  });

  it("snaps open when swiped left past half total-reveal width", async () => {
    const onAction = vi.fn();
    const { container } = render(() => (
      <SwipeableRow
        actions={[{ label: "Archive", color: "bg-blue-500", onAction }]}
        actionWidth={72}
      >
        <span>swipeable</span>
      </SwipeableRow>
    ));

    const row = container.firstElementChild as HTMLElement;
    // Swipe left by 50px (> 72/2 = 36px threshold).
    fireSwipe(row, -50);

    // After snapping, the inner content div should be translated.
    // We verify by checking that the transform was applied (style attribute).
    const contentDiv = row.querySelector<HTMLElement>('[style*="transform"]');
    expect(contentDiv).not.toBeNull();
  });
});
