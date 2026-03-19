import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@solidjs/testing-library";
import userEvent from "@testing-library/user-event";
import { PullToRefresh } from "~/components/mobile/PullToRefresh";

// Helper: fire a complete touch gesture on an element.
function fireTouchGesture(
  element: HTMLElement,
  startY: number,
  moveY: number,
) {
  const touchStart = new TouchEvent("touchstart", {
    bubbles: true,
    touches: [new Touch({ identifier: 1, target: element, clientY: startY, clientX: 0 })],
  });
  const touchMove = new TouchEvent("touchmove", {
    bubbles: true,
    cancelable: true,
    touches: [new Touch({ identifier: 1, target: element, clientY: moveY, clientX: 0 })],
  });
  const touchEnd = new TouchEvent("touchend", {
    bubbles: true,
    changedTouches: [new Touch({ identifier: 1, target: element, clientY: moveY, clientX: 0 })],
  });

  element.dispatchEvent(touchStart);
  element.dispatchEvent(touchMove);
  element.dispatchEvent(touchEnd);
}

describe("PullToRefresh", () => {
  it("renders its children", () => {
    render(() => (
      <PullToRefresh onRefresh={vi.fn()} refreshing={false}>
        <p>child content</p>
      </PullToRefresh>
    ));
    expect(screen.getByText("child content")).toBeInTheDocument();
  });

  it("calls onRefresh after pulling past threshold", async () => {
    const onRefresh = vi.fn().mockResolvedValue(undefined);

    render(() => (
      <PullToRefresh onRefresh={onRefresh} refreshing={false} threshold={72}>
        <p>content</p>
      </PullToRefresh>
    ));

    const container = screen.getByText("content").parentElement!;

    // Simulate a pull from y=0 to y=200 (well past the 72px threshold).
    fireTouchGesture(container, 0, 200);

    // onRefresh is called asynchronously from onTouchEnd.
    await vi.waitFor(() => expect(onRefresh).toHaveBeenCalledOnce());
  });

  it("does not call onRefresh when refreshing is already true", async () => {
    const onRefresh = vi.fn();

    render(() => (
      <PullToRefresh onRefresh={onRefresh} refreshing={true} threshold={72}>
        <p>content</p>
      </PullToRefresh>
    ));

    const container = screen.getByText("content").parentElement!;
    fireTouchGesture(container, 0, 200);

    // Give any async microtasks a chance to run.
    await new Promise((r) => setTimeout(r, 50));
    expect(onRefresh).not.toHaveBeenCalled();
  });

  it("does not call onRefresh when pull distance is below threshold", async () => {
    const onRefresh = vi.fn();

    render(() => (
      <PullToRefresh onRefresh={onRefresh} refreshing={false} threshold={72}>
        <p>content</p>
      </PullToRefresh>
    ));

    const container = screen.getByText("content").parentElement!;
    // Pull only 20px — not enough to trigger.
    fireTouchGesture(container, 0, 20);

    await new Promise((r) => setTimeout(r, 50));
    expect(onRefresh).not.toHaveBeenCalled();
  });
});
