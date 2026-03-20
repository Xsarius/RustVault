/**
 * Demo mode — simulated network latency.
 *
 * Wraps any value in a Promise that resolves after a realistic
 * 50–300ms random delay, mimicking a real API response.
 */

/** Returns a Promise that resolves with `data` after a random delay. */
export function simulate<T>(data: T): Promise<T> {
  return new Promise((resolve) =>
    setTimeout(() => resolve(data), 50 + Math.random() * 250),
  );
}
