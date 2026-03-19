/**
 * Capacitor plugin mocks for testing.
 *
 * Capacitor plugins use native bridges unavailable in jsdom.
 * These stubs replace them during unit / component tests.
 */

import { vi } from "vitest";

export const mockPreferencesStore: Record<string, string> = {};

export const Preferences = {
  get: vi.fn(async ({ key }: { key: string }) => ({
    value: mockPreferencesStore[key] ?? null,
  })),
  set: vi.fn(async ({ key, value }: { key: string; value: string }) => {
    mockPreferencesStore[key] = value;
  }),
  remove: vi.fn(async ({ key }: { key: string }) => {
    delete mockPreferencesStore[key];
  }),
  keys: vi.fn(async () => ({ keys: Object.keys(mockPreferencesStore) })),
};

export const Network = {
  getStatus: vi.fn(async () => ({ connected: true, connectionType: "wifi" })),
  addListener: vi.fn(async () => ({ remove: vi.fn() })),
};

export const Camera = {
  getPhoto: vi.fn(async () => ({
    base64String: "dGVzdA==",
    format: "jpeg",
  })),
};

export const Share = {
  share: vi.fn(async () => {}),
};

export const Filesystem = {
  writeFile: vi.fn(async () => ({ uri: "file:///tmp/test.csv" })),
  deleteFile: vi.fn(async () => {}),
};

export const Device = {
  getLanguageTag: vi.fn(async () => ({ value: "en-US" })),
};

/** Reset all mock implementations and call history between tests. */
export function resetCapacitorMocks() {
  vi.clearAllMocks();
  Object.keys(mockPreferencesStore).forEach((k) => delete mockPreferencesStore[k]);

  Preferences.get.mockImplementation(async ({ key }: { key: string }) => ({
    value: mockPreferencesStore[key] ?? null,
  }));
  Preferences.set.mockImplementation(async ({ key, value }: { key: string; value: string }) => {
    mockPreferencesStore[key] = value;
  });
  Preferences.remove.mockImplementation(async ({ key }: { key: string }) => {
    delete mockPreferencesStore[key];
  });
  Preferences.keys.mockImplementation(async () => ({
    keys: Object.keys(mockPreferencesStore),
  }));

  Network.getStatus.mockResolvedValue({ connected: true, connectionType: "wifi" });
  Network.addListener.mockResolvedValue({ remove: vi.fn() });
}
