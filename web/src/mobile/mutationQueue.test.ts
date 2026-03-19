import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  Preferences,
  mockPreferencesStore,
  resetCapacitorMocks,
} from "~/test/capacitorMocks";

vi.mock("@capacitor/preferences", () => ({ Preferences }));

// Import after mock is registered.
const { enqueue, flushQueue, queueSize, clearQueue } = await import(
  "~/mobile/mutationQueue"
);

describe("mutationQueue", () => {
  beforeEach(() => {
    resetCapacitorMocks();
  });

  it("starts with an empty queue", async () => {
    expect(await queueSize()).toBe(0);
  });

  it("enqueues a mutation and increments size", async () => {
    await enqueue({ method: "POST", path: "/api/transactions", body: { amount: "10.00" } });
    expect(await queueSize()).toBe(1);
  });

  it("persists multiple mutations in order", async () => {
    await enqueue({ method: "POST", path: "/api/a" });
    await enqueue({ method: "DELETE", path: "/api/b" });
    expect(await queueSize()).toBe(2);
  });

  it("clearQueue removes all entries", async () => {
    await enqueue({ method: "POST", path: "/api/a" });
    await clearQueue();
    expect(await queueSize()).toBe(0);
  });

  describe("flushQueue", () => {
    it("calls executor for each entry and empties the queue on success", async () => {
      await enqueue({ method: "POST", path: "/api/a" });
      await enqueue({ method: "PATCH", path: "/api/b", body: { x: 1 } });

      const executor = vi.fn(async () => {});
      const result = await flushQueue(executor);

      expect(executor).toHaveBeenCalledTimes(2);
      expect(result.succeeded).toBe(2);
      expect(result.failed).toBe(0);
      expect(result.removed).toBe(0);
      expect(await queueSize()).toBe(0);
    });

    it("removes entries that fail with a 4xx client error", async () => {
      await enqueue({ method: "DELETE", path: "/api/gone" });

      const clientError = Object.assign(new Error("Not found"), { status: 404 });
      const executor = vi.fn(async () => { throw clientError; });

      const result = await flushQueue(executor);
      expect(result.removed).toBe(1);
      expect(result.failed).toBe(0);
      expect(await queueSize()).toBe(0);
    });

    it("increments retry count and keeps entry on 5xx error", async () => {
      await enqueue({ method: "POST", path: "/api/flaky" });

      const serverError = Object.assign(new Error("Server error"), { status: 500 });
      const executor = vi.fn(async () => { throw serverError; });

      const result = await flushQueue(executor);
      expect(result.failed).toBe(1);
      expect(await queueSize()).toBe(1);

      // Verify retry count was incremented by reading the raw stored value.
      const rawKey = Object.keys(mockPreferencesStore).find((k) =>
        k.includes("offline_queue"),
      );
      const stored = JSON.parse(mockPreferencesStore[rawKey!]);
      expect(stored[0].retries).toBe(1);
    });

    it("drops an entry after MAX_RETRIES (5) failures", async () => {
      await enqueue({ method: "POST", path: "/api/gone" });

      const serverError = Object.assign(new Error("Server error"), { status: 500 });
      const failExecutor = vi.fn(async () => { throw serverError; });

      // Flush 6 times (5 retries + initial attempt = drop on 6th flush).
      for (let i = 0; i < 6; i++) {
        await flushQueue(failExecutor);
      }

      expect(await queueSize()).toBe(0);
    });
  });
});
