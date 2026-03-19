import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@solidjs/testing-library";
import { Router, Route } from "@solidjs/router";
import { BottomTabBar } from "~/components/layout/BottomTabBar";

vi.mock("~/i18n", () => ({
  useI18n: () => (key: string) => key,
}));

function renderWithRouter() {
  return render(() => (
    <Router url="/">
      <Route path="*" component={BottomTabBar} />
    </Router>
  ));
}

describe("BottomTabBar", () => {
  it("renders the main navigation tabs", () => {
    renderWithRouter();
    // The nav element should be in the document.
    expect(document.querySelector("nav")).toBeInTheDocument();
  });

  it("renders the '+' action button", () => {
    renderWithRouter();
    // The Plus button is a <button> inside the nav — it opens the action sheet.
    const buttons = screen.getAllByRole("button");
    // At least the plus (+) button should exist.
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("opens the action sheet when the '+' button is clicked", async () => {
    renderWithRouter();
    // Before clicking, the action sheet should not be visible.
    expect(screen.queryByText("common.mobile.quickActions")).toBeNull();

    // The '+' button is the first (and only initially visible) <button> in the nav.
    const plusButton = document.querySelector("nav button")!;
    (plusButton as HTMLElement).click();

    expect(
      await screen.findByText("common.mobile.quickActions"),
    ).toBeInTheDocument();
  });

  it("closes the action sheet when the backdrop is clicked", async () => {
    renderWithRouter();

    // Open the sheet.
    (document.querySelector("nav button") as HTMLElement).click();
    await screen.findByText("common.mobile.quickActions");

    // The backdrop is the first fixed-overlay div rendered by the Show block.
    const backdrop = document.querySelector(".fixed.inset-0") as HTMLElement;
    backdrop.click();

    // Sheet should be gone.
    expect(screen.queryByText("common.mobile.quickActions")).toBeNull();
  });

  it("closes the action sheet when the X button is clicked", async () => {
    renderWithRouter();

    (document.querySelector("nav button") as HTMLElement).click();
    await screen.findByText("common.mobile.quickActions");

    // The X (close) button has a unique "text-text-tertiary" class inside the sheet.
    const xButton = document.querySelector<HTMLElement>("button.text-text-tertiary");
    expect(xButton).not.toBeNull();
    xButton!.click();

    expect(screen.queryByText("common.mobile.quickActions")).toBeNull();
  });

  it("action items are visible inside the open sheet", async () => {
    renderWithRouter();
    (document.querySelector("nav button") as HTMLElement).click();
    await screen.findByText("common.mobile.quickActions");

    // At least the first action key should be rendered.
    expect(screen.getByText("common.mobile.importFile")).toBeInTheDocument();
  });
});
