/**
 * Locale-aware formatting utilities for amounts and dates.
 *
 * All functions accept an optional `locale` string (BCP 47, e.g. "en-US",
 * "de-DE"). When omitted they fall back to `navigator.language` so the output
 * matches the user's OS preference automatically.
 */

// ── Currency / amount formatting ─────────────────────────────

/**
 * Format a decimal string or number as a currency value using `Intl.NumberFormat`.
 *
 * Examples (locale "en-US", currency "EUR"):  1234.5  →  "€1,234.50"
 * Examples (locale "de-DE", currency "EUR"):  1234.5  →  "1.234,50 €"
 *
 * @param value   - Decimal string from the API (e.g. "1234.5600") or a number.
 * @param currency - ISO 4217 currency code, e.g. "EUR", "USD".
 * @param locale  - BCP 47 locale code. Defaults to `navigator.language`.
 */
export function formatCurrency(
  value: string | number,
  currency: string,
  locale?: string,
): string {
  const n = typeof value === "string" ? parseFloat(value) : value;
  if (isNaN(n)) return String(value);
  const effectiveLocale = locale ?? (typeof navigator !== "undefined" ? navigator.language : "en-US");
  try {
    return new Intl.NumberFormat(effectiveLocale, {
      style: "currency",
      currency,
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(n);
  } catch {
    // Fallback for unknown currency codes (e.g. crypto tickers)
    return new Intl.NumberFormat(effectiveLocale, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(n) + " " + currency;
  }
}

/**
 * Format a decimal string or number as a plain localized number (no currency symbol).
 *
 * Useful for table cells where the currency symbol is already shown in the header.
 */
export function formatAmount(
  value: string | number,
  fractionDigits = 2,
  locale?: string,
): string {
  const n = typeof value === "string" ? parseFloat(value) : value;
  if (isNaN(n)) return String(value);
  const effectiveLocale = locale ?? (typeof navigator !== "undefined" ? navigator.language : "en-US");
  return new Intl.NumberFormat(effectiveLocale, {
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  }).format(n);
}

// ── Date formatting ───────────────────────────────────────────

/**
 * Format an ISO date string (YYYY-MM-DD) for display using `Intl.DateTimeFormat`.
 *
 * Examples (locale "en-US"):  "2026-03-01"  →  "Mar 1, 2026"
 * Examples (locale "de-DE"):  "2026-03-01"  →  "1. März 2026"
 *
 * @param isoDate - ISO 8601 date string, e.g. "2026-03-01".
 * @param locale  - BCP 47 locale code. Defaults to `navigator.language`.
 * @param options - `Intl.DateTimeFormatOptions` overrides.
 */
export function formatDate(
  isoDate: string,
  locale?: string,
  options: Intl.DateTimeFormatOptions = { year: "numeric", month: "short", day: "numeric" },
): string {
  // Parse as UTC midnight to avoid timezone-shift surprises
  const [year, month, day] = isoDate.split("-").map(Number);
  const d = new Date(Date.UTC(year, month - 1, day));
  if (isNaN(d.getTime())) return isoDate;
  const effectiveLocale = locale ?? (typeof navigator !== "undefined" ? navigator.language : "en-US");
  return new Intl.DateTimeFormat(effectiveLocale, { ...options, timeZone: "UTC" }).format(d);
}

/**
 * Format a budget period as "MonthName YYYY" derived from its start date.
 *
 * Examples (locale "en-US"):  "2026-03-01"  →  "March 2026"
 * Examples (locale "de-DE"):  "2026-03-01"  →  "März 2026"
 */
export function formatBudgetPeriodLabel(
  periodStart: string,
  locale?: string,
): string {
  return formatDate(periodStart, locale, { year: "numeric", month: "long" });
}

/**
 * Format a budget date range as "StartDate – EndDate".
 *
 * Examples (locale "en-US"):  "2026-03-01" / "2026-03-31"  →  "Mar 1 – Mar 31, 2026"
 */
export function formatDateRange(
  start: string,
  end: string,
  locale?: string,
): string {
  const effectiveLocale = locale ?? (typeof navigator !== "undefined" ? navigator.language : "en-US");
  const [sy, sm, sd] = start.split("-").map(Number);
  const [ey, em, ed] = end.split("-").map(Number);
  const startDate = new Date(Date.UTC(sy, sm - 1, sd));
  const endDate = new Date(Date.UTC(ey, em - 1, ed));
  if (isNaN(startDate.getTime()) || isNaN(endDate.getTime())) return `${start} – ${end}`;
  try {
    const fmt = new Intl.DateTimeFormat(effectiveLocale, {
      year: "numeric",
      month: "short",
      day: "numeric",
      timeZone: "UTC",
    });
    return fmt.formatRange(startDate, endDate);
  } catch {
    // formatRange not available in all environments — fallback
    const short = (d: Date) =>
      new Intl.DateTimeFormat(effectiveLocale, { month: "short", day: "numeric", timeZone: "UTC" }).format(d);
    const year = new Intl.DateTimeFormat(effectiveLocale, { year: "numeric", timeZone: "UTC" }).format(endDate);
    return `${short(startDate)} – ${short(endDate)}, ${year}`;
  }
}
