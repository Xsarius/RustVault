# Reports & Charts

RustVault provides interactive financial reports and visualisations. All reports
are scoped to the authenticated user and support date range selection.

## Dashboard

The dashboard gives you a live snapshot the moment you log in:

| Widget | What it shows |
|--------|--------------|
| **Net Worth** | Sum of all non-archived account balances. Hover the `?` for a description. |
| **Income (month)** | All income transactions in the current calendar month. |
| **Expenses (month)** | All expense transactions in the current calendar month. |
| **Savings Rate** | `(Income − Expenses) ÷ Income × 100`. Blank when income is zero. |
| **Monthly Trend** | Bar chart of income vs. expenses for the last 12 months. |
| **Spending by Category** | Donut chart of your top spending categories this month. |
| **Unreviewed badge** | Quick link to transactions that still need your review. |

The dashboard is loaded via `GET /api/reports/summary`.

## Reports page

Navigate to **Reports** in the sidebar to access four analytical tabs.

### Income vs. Expense

A stacked bar chart showing monthly income and expenses with a net cash-flow
line overlay. Powered by `GET /api/reports/income-expense`.

- **Date range** — pick any custom range or choose a preset (3, 6, or 12 months).
- **Export** — click **Export CSV** to download the monthly breakdown.

### Category Trend

Tracks how spending in a single category changes month over month.
Powered by `GET /api/reports/categories/:id/trend`.

1. Select a category from the dropdown.
2. Choose a date range.
3. A bar chart shows spending per month; a dashed line marks the simple average.
4. Export the data as CSV.

### Balance History

Line charts showing the running balance for each account and a combined net
worth line. Powered by `GET /api/reports/balance-history`.

- Balances are reconstructed from the current cached balance by replaying
  transactions backwards.
- When more than 500 data points are returned the backend applies the
  **LTTB** (Largest-Triangle-Three-Buckets) downsampling algorithm to keep
  the chart responsive while preserving the visual shape.
- Export the snapshots as CSV.

### Cash Flow

Monthly income and expenses with a 3-month forecast appended. Forecast periods
are shown at reduced opacity. Powered by `GET /api/reports/cash-flow`.

The forecast is computed as the simple average of income and expenses across
the selected historical range.

- Export the full data (history + forecast) as CSV.

## Date range selection

All report tabs share the same date range controls:

- **Custom** — type any `from` and `to` dates and press **Apply**.
- **Presets** — one-click buttons for Last 3 months, Last 6 months, Last 12 months.

Dates are passed to the API as query parameters in `YYYY-MM-DD` format.

## Export

Every report tab has an **Export CSV** button that appears once data has loaded.
The downloaded file contains the same data shown in the chart. Column names and
values are in English regardless of locale.

## Performance

- All chart types (bar, line, pie) are loaded from **modular ECharts** — only the
  chart types and components actually used are imported.
- Charts are rendered on a **Canvas** renderer for hardware acceleration.
- The `lib/chart.ts` utility registers the user’s locale with ECharts so axis
  labels and tooltips respect locale-specific number and month formatting.
- A `ResizeObserver` ensures charts reflow correctly when the viewport changes.
