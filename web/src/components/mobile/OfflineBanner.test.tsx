import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@solidjs/testing-library";
import { OfflineBanner } from "~/components/mobile/OfflineBanner";

// Return the key name so we can assert translated strings without a real dict.
vi.mock("~/i18n", () => ({
  useI18n: () => (key: string) => key,
}));

describe("OfflineBanner", () => {
  it("shows the offline warning when isOnline=false", () => {
    render(() => (
      <OfflineBanner
        isOnline={false}
        isSyncing={false}
        pendingCount={0}
        onSync={vi.fn()}
      />
    ));
    // The offline message key should be present.
    expect(screen.getByText("common.mobile.offline")).toBeInTheDocument();
  });

  it("shows pending count badge when offline with queued mutations", () => {
    render(() => (
      <OfflineBanner
        isOnline={false}
        isSyncing={false}
        pendingCount={3}
        onSync={vi.fn()}
      />
    ));
    expect(screen.getByText("common.mobile.offline")).toBeInTheDocument();
    // The pending count and label should be rendered.
    expect(screen.getByText(/3/)).toBeInTheDocument();
  });

  it("shows the syncing banner when online and syncing", () => {
    render(() => (
      <OfflineBanner
        isOnline={true}
        isSyncing={true}
        pendingCount={2}
        onSync={vi.fn()}
      />
    ));
    expect(screen.getByText("common.mobile.syncing")).toBeInTheDocument();
  });

  it("shows the sync-ready banner when online, not syncing, with pending items", () => {
    render(() => (
      <OfflineBanner
        isOnline={true}
        isSyncing={false}
        pendingCount={5}
        onSync={vi.fn()}
      />
    ));
    expect(screen.getByText("common.mobile.syncReady")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "common.mobile.syncNow" }),
    ).toBeInTheDocument();
  });

  it("calls onSync when the sync button is clicked", async () => {
    const onSync = vi.fn();
    render(() => (
      <OfflineBanner
        isOnline={true}
        isSyncing={false}
        pendingCount={1}
        onSync={onSync}
      />
    ));
    screen.getByRole("button", { name: "common.mobile.syncNow" }).click();
    expect(onSync).toHaveBeenCalledOnce();
  });

  it("renders nothing visible when fully online with no pending items", () => {
    const { container } = render(() => (
      <OfflineBanner
        isOnline={true}
        isSyncing={false}
        pendingCount={0}
        onSync={vi.fn()}
      />
    ));
    // All three Show conditions are false — the container should be empty (or
    // contain only an empty fragment comment nodes with no visible elements).
    expect(container.querySelector('[class*="fixed"]')).toBeNull();
  });
});
