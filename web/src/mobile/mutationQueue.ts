/**
 * mutationQueue — offline mutation queue backed by @capacitor/preferences.
 *
 * When the device is offline, write mutations (POST / PUT / PATCH / DELETE)
 * are serialized to a persistent queue. When connectivity is restored the
 * queue is replayed in order against the live API.
 *
 * Design decisions:
 * - The queue is persisted to device storage so it survives app restarts.
 * - Each entry has a unique ID, timestamp, and retry count.
 * - Entries that fail with a 4xx are removed (they will never succeed).
 * - Entries that fail with a 5xx or network error are retried up to MAX_RETRIES.
 * - The queue is replayed in FIFO order.
 */

import { Preferences } from "@capacitor/preferences";

const QUEUE_KEY = "rustvault.offline_queue";
const MAX_RETRIES = 5;

export type HttpMethod = "POST" | "PUT" | "PATCH" | "DELETE";

export interface QueuedMutation {
  id: string;
  method: HttpMethod;
  path: string;
  body?: unknown;
  retries: number;
  createdAt: number;
}

// ── Persistence helpers ───────────────────────────────────────────────────────

async function readQueue(): Promise<QueuedMutation[]> {
  try {
    const { value } = await Preferences.get({ key: QUEUE_KEY });
    return value ? (JSON.parse(value) as QueuedMutation[]) : [];
  } catch {
    return [];
  }
}

async function writeQueue(queue: QueuedMutation[]): Promise<void> {
  await Preferences.set({ key: QUEUE_KEY, value: JSON.stringify(queue) });
}

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Append a mutation to the offline queue.
 *
 * Call this when a fetch fails because the device is offline.
 */
export async function enqueue(mutation: Omit<QueuedMutation, "id" | "retries" | "createdAt">): Promise<void> {
  const queue = await readQueue();
  queue.push({
    ...mutation,
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
    retries: 0,
    createdAt: Date.now(),
  });
  await writeQueue(queue);
}

/** Return number of pending mutations. */
export async function queueSize(): Promise<number> {
  const queue = await readQueue();
  return queue.length;
}

/** Clear the entire queue (e.g. on logout). */
export async function clearQueue(): Promise<void> {
  await Preferences.remove({ key: QUEUE_KEY });
}

/**
 * Replay all queued mutations in order.
 *
 * `executor` is called with each mutation and should throw on failure.
 * Returns counts of { succeeded, failed, removed }.
 */
export async function flushQueue(
  executor: (mutation: QueuedMutation) => Promise<void>,
): Promise<{ succeeded: number; failed: number; removed: number }> {
  const queue = await readQueue();
  if (queue.length === 0) return { succeeded: 0, failed: 0, removed: 0 };

  const remaining: QueuedMutation[] = [];
  let succeeded = 0;
  let failed = 0;
  let removed = 0;

  for (const entry of queue) {
    try {
      await executor(entry);
      succeeded++;
      // Entry removed from queue on success.
    } catch (err: unknown) {
      const isClientError =
        err instanceof Error &&
        "status" in err &&
        typeof (err as { status: unknown }).status === "number" &&
        (err as { status: number }).status >= 400 &&
        (err as { status: number }).status < 500;

      if (isClientError || entry.retries >= MAX_RETRIES) {
        // 4xx errors or exhausted retries — drop the entry.
        removed++;
      } else {
        remaining.push({ ...entry, retries: entry.retries + 1 });
        failed++;
      }
    }
  }

  await writeQueue(remaining);
  return { succeeded, failed, removed };
}
