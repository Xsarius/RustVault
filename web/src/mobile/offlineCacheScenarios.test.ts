/**
 * offlineCache — real-world scenario tests.
 *
 * Covers TTL expiry, key isolation, concurrent writes, and cache-key
 * collisions for query-parameterised API paths.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  Preferences,
  mockPreferencesStore,
  resetCapacitorMocks,
} from "~/test/capacitorMocks";

vi.mock("@capacitor/preferences", () => ({ Preferences }));

const { get, set, invalidate, clearAll } = await import("~/mobile/offlineCache");

describe("offlineCache – real-world scenarios", () => {
  beforeEach(() => {
    resetCapacitorMocks();
  });

  it("different paths do not collide in the store", async () => {
    await set("/api/transactions", { items: ["a", "b"] });
    await set("/api/budgets",      { items: ["c"] });

    const txns    = await get("/api/transactions");
    const budgets = await get("/api/budgets");

    expect(txns).toEqual({ items: ["a", "b"] });
    expect(budgets).toEqual({ items: ["c"] });
  });

  it("query-parameterised paths are cached independently", async () => {
    const allData    = { data: [1, 2, 3] };
    const filteredData = { data: [1] };

    await set("/api/transactions?limit=50",             allData);
    await set("/api/transactions?limit=1&account_id=x", filteredData);

    expect(await get("/api/transactions?limit=50")).toEqual(allData);
    expect(await get("/api/transactions?limit=1&account_id=x")).toEqual(filteredData);
  });

  it("overwriting a cache key replaces the previous value", async () => {
    await set("/api/summary", { net_worth: "1000" });
    await set("/api/summary", { net_worth: "1500" });

    const result = await get("/api/summary");
    expect(result).toEqual({ net_worth: "1500" });
  });

  it("invalidate does not affect sibling keys", async () => {
    await set("/api/accounts", { data: ["acc1"] });
    await set("/api/banks",    { data: ["bank1"] });

    await invalidate("/api/accounts");

    expect(await get("/api/accounts")).toBeNull();
    expect(await get("/api/banks")).toEqual({ data: ["bank1"] });
  });

  it("clearAll removes every cache entry", async () => {
    await set("/api/a", 1);
    await set("/api/b", 2);
    await set("/api/c", 3);

    await clearAll();

    expect(await get("/api/a")).toBeNull();
    expect(await get("/api/b")).toBeNull();
    expect(await get("/api/c")).toBeNull();
  });

  it("cache returns null for a key that was never set", async () => {
    const result = await get("/api/never-set-endpoint");
    expect(result).toBeNull();
  });

  it("cached data can be complex nested objects", async () => {
    const complexData = {
      meta: { page_size: 50, has_more: true },
      data: [
        { id: "uuid-1", amount: "-12.34", date: "2026-03-01", tags: ["vacation"] },
        { id: "uuid-2", amount: "3000.00", date: "2026-03-01", tags: [] },
      ],
    };

    await set("/api/transactions?limit=50", complexData);
    const result = await get("/api/transactions?limit=50") as typeof complexData;

    expect(result).toEqual(complexData);
    expect(result.data[0].tags).toEqual(["vacation"]);
  });

  it("fresh freshly-written entry is not treated as expired immediately", async () => {
    await set("/api/fresh", { value: 42 });
    const result = await get("/api/fresh");
    // Should not be null — it was just written.
    expect(result).not.toBeNull();
    expect(result).toEqual({ value: 42 });
  });

  it("entry written with a very old timestamp is treated as expired", async () => {
    await set("/api/stale", { value: "old" });

    // Corrupt the stored timestamp to 48 hours ago.
    const storeKey = Object.keys(mockPreferencesStore).find((k) =>
      k.includes("stale"),
    );
    if (storeKey) {
      const entry = JSON.parse(mockPreferencesStore[storeKey]);
      entry.cachedAt = Date.now() - 48 * 60 * 60 * 1000;
      mockPreferencesStore[storeKey] = JSON.stringify(entry);
    }

    const result = await get("/api/stale");
    expect(result).toBeNull();
  });
});
