/**
 * offlineCache — lightweight read-through cache for offline support.
 *
 * Stores recent API GET responses in @capacitor/preferences so the app
 * can display stale data when the network is unavailable.
 *
 * Cache keys mirror the API path. Entries are stored with a timestamp;
 * stale entries older than MAX_AGE_MS are evicted lazily.
 *
 * Usage (in API layer):
 *   const data = await offlineCache.get<TxList>("/api/transactions?limit=50");
 *   if (!data) fetchFromNetwork();
 *
 *   offlineCache.set("/api/transactions?limit=50", freshData);
 */

import { Preferences } from "@capacitor/preferences";

const CACHE_PREFIX = "rustvault.cache.";

/** Maximum cache entry age: 24 hours. */
const MAX_AGE_MS = 24 * 60 * 60 * 1000;

interface CacheEntry<T> {
  data: T;
  cachedAt: number;
}

function cacheKey(path: string): string {
  // Sanitize the path to a safe key by replacing non-alphanumeric chars.
  return CACHE_PREFIX + path.replace(/[^a-z0-9]/gi, "_");
}

/** Retrieve a cached response. Returns null on miss or expiry. */
export async function get<T>(path: string): Promise<T | null> {
  try {
    const { value } = await Preferences.get({ key: cacheKey(path) });
    if (!value) return null;

    const entry = JSON.parse(value) as CacheEntry<T>;
    if (Date.now() - entry.cachedAt > MAX_AGE_MS) {
      // Evict expired entry.
      await Preferences.remove({ key: cacheKey(path) });
      return null;
    }

    return entry.data;
  } catch {
    return null;
  }
}

/** Store a response for the given path. */
export async function set<T>(path: string, data: T): Promise<void> {
  try {
    const entry: CacheEntry<T> = { data, cachedAt: Date.now() };
    await Preferences.set({ key: cacheKey(path), value: JSON.stringify(entry) });
  } catch {
    // Storage full or unavailable — ignore.
  }
}

/** Remove a specific cached entry (e.g. after a mutation). */
export async function invalidate(path: string): Promise<void> {
  try {
    await Preferences.remove({ key: cacheKey(path) });
  } catch {
    // Ignore.
  }
}

/** Clear all cached entries (e.g. on logout). */
export async function clearAll(): Promise<void> {
  try {
    const { keys } = await Preferences.keys();
    await Promise.all(
      keys
        .filter((k) => k.startsWith(CACHE_PREFIX))
        .map((k) => Preferences.remove({ key: k })),
    );
  } catch {
    // Ignore.
  }
}

export const offlineCache = { get, set, invalidate, clearAll };
