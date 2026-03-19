/**
 * useOfflineSync — orchestrates offline detection, queue replay, and cache.
 *
 * Mount this once near the app root. It:
 * 1. Tracks network status reactively.
 * 2. When coming back online, flushes the mutation queue via the API client.
 * 3. Exposes `isOnline` and `pendingCount` to the UI.
 */

import { createSignal, createEffect, on, onMount } from "solid-js";
import { useNetworkStatus } from "./useNetworkStatus";
import { flushQueue, queueSize, type QueuedMutation } from "./mutationQueue";
import * as api from "~/api/client";

export interface OfflineSyncState {
  isOnline: () => boolean;
  isSyncing: () => boolean;
  pendingCount: () => number;
  /** Manually trigger a sync (called after reconnect or by user). */
  sync: () => Promise<void>;
}

export function useOfflineSync(): OfflineSyncState {
  const networkStatus = useNetworkStatus();
  const [isSyncing, setIsSyncing] = createSignal(false);
  const [pendingCount, setPendingCount] = createSignal(0);

  const isOnline = () => networkStatus().online;

  // Refresh the pending count on mount.
  onMount(async () => {
    setPendingCount(await queueSize());
  });

  // When we transition from offline → online, flush the queue.
  createEffect(
    on(
      isOnline,
      async (online, prevOnline) => {
        if (online && prevOnline === false) {
          await sync();
        }
      },
      { defer: true },
    ),
  );

  async function sync() {
    const current = await queueSize();
    if (current === 0) return;

    setIsSyncing(true);
    setPendingCount(current);

    await flushQueue(async (mutation: QueuedMutation) => {
      switch (mutation.method) {
        case "POST":
          await api.post(mutation.path, mutation.body);
          break;
        case "PUT":
          await api.put(mutation.path, mutation.body);
          break;
        case "PATCH":
          await api.patch(mutation.path, mutation.body);
          break;
        case "DELETE":
          await api.del(mutation.path);
          break;
      }
    });

    setPendingCount(await queueSize());
    setIsSyncing(false);
  }

  return { isOnline, isSyncing, pendingCount, sync };
}
