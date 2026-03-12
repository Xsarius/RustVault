/// <reference types="vite/client" />
/**
 * Performance monitoring — web-vitals integration.
 *
 * Measures Core Web Vitals (CLS, FID, FCP, LCP, TTFB) and logs them.
 * In production, these could be sent to an analytics endpoint.
 */

import type { Metric } from "web-vitals";

function reportMetric(metric: Metric) {
  // In development, log to console
  if (import.meta.env.DEV) {
    console.debug(
      `[vitals] ${metric.name}: ${metric.value.toFixed(2)} (${metric.rating})`,
    );
  }

  // TODO: In production, send to /api/metrics or analytics service
  // api.post("/api/metrics/vital", { name: metric.name, value: metric.value, rating: metric.rating });
}

/**
 * Initialize web-vitals reporting.
 * Call once at app startup. Uses dynamic import so web-vitals
 * is only loaded when needed.
 */
export async function initVitals() {
  const { onCLS, onFCP, onLCP, onTTFB } = await import("web-vitals");

  onCLS(reportMetric);
  onFCP(reportMetric);
  onLCP(reportMetric);
  onTTFB(reportMetric);
}
