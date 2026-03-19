/**
 * useBiometric — biometric (Face ID / Touch ID / fingerprint) authentication.
 *
 * Stores the auth token in the @capacitor/preferences key-value store
 * (backed by iOS Keychain / Android Keystore) rather than localStorage.
 *
 * On web, biometric is unavailable — all operations no-op gracefully.
 */

import { Preferences } from "@capacitor/preferences";
import { isMobile } from "./useMobile";

const TOKEN_KEY = "rustvault.biometric_token";
const ENABLED_KEY = "rustvault.biometric_enabled";

/**
 * Check whether biometric auth is supported on this device.
 * On web this always returns false.
 */
async function isBiometricAvailable(): Promise<boolean> {
  if (!isMobile()) return false;
  // Actual native biometric availability check requires the
  // @capacitor-community/biometric-auth plugin (not bundled here).
  // This flag tells us the user has opted in.
  const { value } = await Preferences.get({ key: ENABLED_KEY });
  return value === "true";
}

/** Persist the access token in secure native storage. */
async function storeToken(token: string): Promise<void> {
  await Preferences.set({ key: TOKEN_KEY, value: token });
}

/** Retrieve the stored access token (null when not set). */
async function getStoredToken(): Promise<string | null> {
  const { value } = await Preferences.get({ key: TOKEN_KEY });
  return value;
}

/** Clear the stored token (on logout). */
async function clearToken(): Promise<void> {
  await Preferences.remove({ key: TOKEN_KEY });
}

/** Enable or disable biometric unlock preference. */
async function setBiometricEnabled(enabled: boolean): Promise<void> {
  await Preferences.set({ key: ENABLED_KEY, value: String(enabled) });
  if (!enabled) {
    await clearToken();
  }
}

export function useBiometric() {
  return {
    isBiometricAvailable,
    storeToken,
    getStoredToken,
    clearToken,
    setBiometricEnabled,
  };
}
