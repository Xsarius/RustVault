/**
 * Mobile / Capacitor integration entrypoint.
 *
 * Re-exports all mobile utility hooks and helpers.
 */

export { useMobile, isPlatform, isMobile } from "./useMobile";
export { useCamera } from "./useCamera";
export { useFilePicker } from "./useFilePicker";
export { useShareFile } from "./useShareFile";
export { useBiometric } from "./useBiometric";
export { usePushNotifications } from "./usePushNotifications";
export { mobileLocale } from "./mobileLocale";
export { useNetworkStatus } from "./useNetworkStatus";
export { enqueue, queueSize, clearQueue, flushQueue } from "./mutationQueue";
export { offlineCache } from "./offlineCache";
export { useOfflineSync } from "./useOfflineSync";
