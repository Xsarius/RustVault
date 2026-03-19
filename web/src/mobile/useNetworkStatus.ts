/**
 * useNetworkStatus — reactive online/offline detection.
 *
 * On Capacitor native the Network plugin provides accurate connectivity info
 * including connection type. On web we fall back to the browser's
 * `navigator.onLine` + online/offline events.
 */

import { createSignal, onCleanup, onMount } from "solid-js";
import { Network } from "@capacitor/network";
import { isMobile } from "./useMobile";

export interface NetworkStatus {
  online: boolean;
  /** "wifi" | "cellular" | "none" | "unknown" */
  connectionType: string;
}

/**
 * Returns a reactive signal containing the current network status.
 * Subscribe to `status().online` in components.
 */
export function useNetworkStatus() {
  const [status, setStatus] = createSignal<NetworkStatus>({
    online: navigator.onLine,
    connectionType: "unknown",
  });

  onMount(async () => {
    if (isMobile()) {
      // Get initial status from the Capacitor plugin.
      try {
        const initial = await Network.getStatus();
        setStatus({
          online: initial.connected,
          connectionType: initial.connectionType,
        });

        const handle = await Network.addListener("networkStatusChange", (conn) => {
          setStatus({
            online: conn.connected,
            connectionType: conn.connectionType,
          });
        });

        onCleanup(() => handle.remove());
      } catch {
        // Plugin unavailable — fall through to browser events.
        attachBrowserListeners();
      }
    } else {
      attachBrowserListeners();
    }
  });

  function attachBrowserListeners() {
    const onOnline = () => setStatus({ online: true, connectionType: "unknown" });
    const onOffline = () => setStatus({ online: false, connectionType: "none" });
    window.addEventListener("online", onOnline);
    window.addEventListener("offline", onOffline);
    onCleanup(() => {
      window.removeEventListener("online", onOnline);
      window.removeEventListener("offline", onOffline);
    });
  }

  return status;
}
