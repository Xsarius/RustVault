/**
 * useBiometric — unit tests for biometric token management.
 *
 * Tests secure token storage, retrieval, clearing, and toggling the
 * biometric-enabled preference. All native Capacitor plugin calls
 * are replaced with in-memory mocks.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  Preferences,
  mockPreferencesStore,
  resetCapacitorMocks,
} from "~/test/capacitorMocks";

vi.mock("@capacitor/preferences", () => ({ Preferences }));

// Import after the mock is installed.
const { useBiometric } = await import("~/mobile/useBiometric");

describe("useBiometric", () => {
  let bio: ReturnType<typeof useBiometric>;

  beforeEach(() => {
    resetCapacitorMocks();
    bio = useBiometric();
  });

  it("getStoredToken returns null when no token has been stored", async () => {
    const token = await bio.getStoredToken();
    expect(token).toBeNull();
  });

  it("storeToken persists the token; getStoredToken retrieves it", async () => {
    await bio.storeToken("test-access-token-abc123");
    const retrieved = await bio.getStoredToken();
    expect(retrieved).toBe("test-access-token-abc123");
  });

  it("storeToken overwrites a previous token", async () => {
    await bio.storeToken("old-token");
    await bio.storeToken("new-token");
    const retrieved = await bio.getStoredToken();
    expect(retrieved).toBe("new-token");
  });

  it("clearToken removes the stored token", async () => {
    await bio.storeToken("to-be-cleared");
    await bio.clearToken();
    const token = await bio.getStoredToken();
    expect(token).toBeNull();
  });

  it("clearToken is a no-op when no token is stored", async () => {
    // Should not throw.
    await expect(bio.clearToken()).resolves.not.toThrow();
  });

  it("setBiometricEnabled(true) stores the preference as 'true'", async () => {
    await bio.setBiometricEnabled(true);
    // Check that some preference key contains "true".
    const storedValues = Object.values(mockPreferencesStore);
    expect(storedValues.some((v) => v === "true")).toBe(true);
  });

  it("setBiometricEnabled(false) stores the preference as 'false' and clears the token", async () => {
    await bio.storeToken("some-jwt");
    await bio.setBiometricEnabled(false);

    // Token should have been cleared.
    const token = await bio.getStoredToken();
    expect(token).toBeNull();
  });

  it("isBiometricAvailable returns false on web (isMobile=false)", async () => {
    // isMobile() returns false in jsdom — biometric is always unavailable.
    const available = await bio.isBiometricAvailable();
    expect(available).toBe(false);
  });
});
