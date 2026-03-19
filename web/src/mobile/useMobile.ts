/**
 * useMobile — detect whether the app is running inside a Capacitor native
 * context (iOS / Android) and expose the current platform.
 */

import { Capacitor } from "@capacitor/core";

/** Returns the current runtime platform: "ios" | "android" | "web" */
export function isPlatform(): "ios" | "android" | "web" {
  return Capacitor.getPlatform() as "ios" | "android" | "web";
}

/** True when running inside a native Capacitor app (not a browser). */
export function isMobile(): boolean {
  return Capacitor.isNativePlatform();
}

/**
 * Solid primitive — returns reactive platform info via a plain getter.
 * Because the platform never changes at runtime, a signal is not needed.
 */
export function useMobile() {
  const platform = isPlatform();
  const native = isMobile();

  return {
    platform,
    isNative: native,
    isIos: platform === "ios",
    isAndroid: platform === "android",
    isWeb: platform === "web",
  };
}
