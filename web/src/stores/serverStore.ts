/**
 * serverStore — persists the self-hosted server URL chosen by the user.
 *
 * On native Capacitor the URL is stored in @capacitor/preferences (backed by
 * iOS Keychain / Android SharedPreferences). On web it falls back to
 * localStorage so the setting survives page refreshes.
 *
 * The store also keeps the API client's base URL in sync so every fetch
 * call is routed to the correct server.
 */

import { createSignal, createRoot } from "solid-js";
import { Preferences } from "@capacitor/preferences";
import { isMobile } from "~/mobile/useMobile";
import { setBaseUrl } from "~/api/client";

const SERVER_URL_KEY = "rustvault.server_url";

function createServerStore() {
  const [serverUrl, setServerUrlSignal] = createSignal<string>("");
  let initialized = false;

  const isConfigured = () => serverUrl().length > 0;

  /**
   * Load the stored URL from persistent storage and push it to the API
   * client. Call this once early in the app lifecycle (e.g. in AuthGuard).
   * Subsequent calls are no-ops.
   */
  async function init(): Promise<void> {
    if (initialized) return;
    initialized = true;

    let url = "";
    if (isMobile()) {
      const { value } = await Preferences.get({ key: SERVER_URL_KEY });
      url = value ?? "";
    } else {
      url = localStorage.getItem(SERVER_URL_KEY) ?? "";
    }

    setServerUrlSignal(url);
    if (url) setBaseUrl(url);
  }

  /** Save a new server URL and update the API base URL immediately. */
  async function setServerUrl(url: string): Promise<void> {
    // Strip trailing slash for consistency.
    const normalized = url.replace(/\/+$/, "");

    if (isMobile()) {
      await Preferences.set({ key: SERVER_URL_KEY, value: normalized });
    } else {
      localStorage.setItem(SERVER_URL_KEY, normalized);
    }

    setServerUrlSignal(normalized);
    setBaseUrl(normalized);
    initialized = true;
  }

  /** Clear the stored URL (e.g. when the user wants to switch servers). */
  async function clearServerUrl(): Promise<void> {
    if (isMobile()) {
      await Preferences.remove({ key: SERVER_URL_KEY });
    } else {
      localStorage.removeItem(SERVER_URL_KEY);
    }

    setServerUrlSignal("");
    setBaseUrl("");
  }

  return {
    /** Reactive getter — current server URL ("" when not configured). */
    serverUrl,
    /** True once the user has saved a server URL. */
    isConfigured,
    init,
    setServerUrl,
    clearServerUrl,
  };
}

/** Singleton server store — created once at module level. */
export const serverStore = createRoot(createServerStore);
