import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  Preferences,
  mockPreferencesStore,
  resetCapacitorMocks,
} from "~/test/capacitorMocks";

vi.mock("@capacitor/preferences", () => ({ Preferences }));

// Import after mock is registered.
const { get, set, invalidate, clearAll } = await import("~/mobile/offlineCache");

describe("offlineCache", () => {
  beforeEach(() => {
    resetCapacitorMocks();
  });

  it("returns null on cache miss", async () => {
    expect(await get("/api/transactions")).toBeNull();
  });

  it("stores and retrieves a value", async () => {
    const data = { transactions: [{ id: "1" }] };
    await set("/api/transactions", data);
    const result = await get("/api/transactions");
    expect(result).toEqual(data);
  });

  it("returns null and evicts an expired entry", async () => {
    const data = { foo: "bar" };
    await set("/api/foo", data);

    // Manually corrupt the cachedAt to something older than 24 h.
    const rawKey = Object.keys(mockPreferencesStore).find((k) =>
      k.includes("api_foo"),
    )!;
    const entry = JSON.parse(mockPreferencesStore[rawKey]);
    entry.cachedAt = Date.now() - 25 * 60 * 60 * 1000;
    mockPreferencesStore[rawKey] = JSON.stringify(entry);

    expect(await get("/api/foo")).toBeNull();
    // Entry should have been removed from the store.
    expect(mockPreferencesStore[rawKey]).toBeUndefined();
  });

  it("invalidate removes the entry for a specific path", async () => {
    await set("/api/bar", { x: 1 });
    await invalidate("/api/bar");
    expect(await get("/api/bar")).toBeNull();
  });

  it("clearAll removes only cache entries", async () => {
    await set("/api/a", 1);
    await set("/api/b", 2);
    // Add a non-cache key manually.
    mockPreferencesStore["rustvault.other"] = "keep";

    await clearAll();

    const remaining = Object.keys(mockPreferencesStore).filter((k) =>
      k.startsWith("rustvault.cache."),
    );
    expect(remaining).toHaveLength(0);
    expect(mockPreferencesStore["rustvault.other"]).toBe("keep");
  });
});
