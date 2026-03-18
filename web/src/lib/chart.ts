/**
 * Modular ECharts loader — lazy-imports only the renderers and components used
 * in RustVault to keep the bundle size manageable.
 *
 * Usage:
 *   import { initChart, type ECharts } from "~/lib/chart";
 *
 *   const chart = await initChart(divElement, "de-DE");
 *   chart.setOption({ ... });
 */

// Re-export the ECharts instance type for use in components.
export type { ECharts } from "echarts/core";

/**
 * Maps a BCP 47 locale tag to an ECharts locale identifier.
 * ECharts ships built-in locales for EN, ZH, DE, FR, JA, ES, PT, etc.
 * Falls back to "EN" when the locale is not natively supported.
 */
function toEChartsLocale(locale: string): string {
  const lang = locale.split("-")[0].toUpperCase();
  const supported = new Set(["EN", "ZH", "DE", "FR", "JA", "ES", "PT", "RU", "PL"]);
  return supported.has(lang) ? lang : "EN";
}

/**
 * Lazily loads the ECharts core with only the modules we need and returns an
 * initialised chart instance attached to `container`.
 *
 * @param container  - DOM element to render into.
 * @param locale     - BCP 47 locale string (e.g. "en-US", "de-DE", "pl-PL").
 *                     Defaults to `navigator.language`.
 *
 * Registered components:
 *  - CanvasRenderer  (hardware-accelerated)
 *  - BarChart, LineChart, PieChart
 *  - GridComponent, TooltipComponent, LegendComponent, TitleComponent
 */
export async function initChart(
  container: HTMLElement,
  locale?: string,
): Promise<import("echarts/core").ECharts> {
  const effectiveLocale = locale ?? (typeof navigator !== "undefined" ? navigator.language : "en-US");
  const echartsLocale = toEChartsLocale(effectiveLocale);

  const [
    { init, use, registerLocale },
    { CanvasRenderer },
    { BarChart, LineChart, PieChart },
    {
      GridComponent,
      TooltipComponent,
      LegendComponent,
      TitleComponent,
    },
  ] = await Promise.all([
    import("echarts/core"),
    import("echarts/renderers"),
    import("echarts/charts"),
    import("echarts/components"),
  ]);

  use([
    CanvasRenderer,
    BarChart,
    LineChart,
    PieChart,
    GridComponent,
    TooltipComponent,
    LegendComponent,
    TitleComponent,
  ]);

  // Register locale data so axis labels, month names, etc. respect the locale.
  // ECharts bundles locale packs under `echarts/lib/i18n/lang*.js`.
  // We lazy-load only the requested locale to avoid bloating the bundle.
  try {
    const localePack = await import(
      /* @vite-ignore */ `echarts/lib/i18n/lang${echartsLocale}`
    );
    registerLocale(echartsLocale, localePack.default ?? localePack);
  } catch {
    // Locale pack not available — ECharts falls back to EN silently.
  }

  return init(container, null, { locale: echartsLocale });
}

/** Common dark/neutral palette shared across all charts. */
export const CHART_COLORS = {
  income: "#22c55e",   // green-500
  expenses: "#ef4444", // red-500
  net: "#3b82f6",      // blue-500
  forecast: "#a855f7", // purple-500
  palette: [
    "#3b82f6",
    "#22c55e",
    "#f59e0b",
    "#ef4444",
    "#8b5cf6",
    "#14b8a6",
    "#f97316",
    "#ec4899",
    "#06b6d4",
    "#84cc16",
  ],
};

/**
 * Build a currency-aware tooltip formatter for ECharts.
 *
 * @param currency - ISO 4217 code (e.g. "USD", "EUR").
 * @param locale   - BCP 47 locale used for `Intl.NumberFormat`.
 */
export function currencyTooltip(currency = "USD", locale = "en-US") {
  const fmt = new Intl.NumberFormat(locale, { style: "currency", currency });
  return {
    trigger: "axis" as const,
    formatter: (params: unknown) => {
      if (!Array.isArray(params)) return "";
      return params
        .map((p: { seriesName?: string; value?: number }) =>
          `${p.seriesName ?? ""}: ${fmt.format(p.value ?? 0)}`,
        )
        .join("<br/>");
    },
  };
}
