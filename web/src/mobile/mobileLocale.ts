/**
 * mobileLocale — detect the device locale from Capacitor on native platforms.
 *
 * On web the browser's `navigator.language` is used.
 * The resolved locale string is passed to the i18n store during app init.
 */

import { Device } from "@capacitor/device";
import { isMobile } from "./useMobile";

/**
 * Resolve the preferred locale for the current device/browser.
 *
 * Returns a BCP 47 locale tag (e.g. "en-US", "pl-PL").
 * Falls back to "en-US" when detection fails.
 */
export async function mobileLocale(): Promise<string> {
  if (isMobile()) {
    try {
      const info = await Device.getLanguageTag();
      return info.value || navigator.language || "en-US";
    } catch {
      // Plugin unavailable — fall through to browser API.
    }
  }

  return navigator.language || navigator.languages?.[0] || "en-US";
}
