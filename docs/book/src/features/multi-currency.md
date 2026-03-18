# Multi-Currency

RustVault supports accounts denominated in different currencies. Each account
has its own ISO 4217 currency (e.g. `EUR`, `USD`, `GBP`, `PLN`).

## How accounts work

When you create an account you must choose its currency. The currency cannot
be changed once transactions exist on the account.

## Exchange rates

RustVault stores exchange rates in the `exchange_rates` table. Rates are fetched
from the **ECB feed** via `POST /api/exchange-rates/refresh`. The stored rates
are used for:

- Calculating approximate cross-currency totals in reports.
- Displaying net worth in your **base currency** (set in Settings).

## Reports & net worth

The dashboard `net_worth` and monthly trend values sum all account balances
**in their native currencies** without conversion. Full multi-currency
conversion in reports (using per-date rates) is planned for a future release.

## Currency formatting

All monetary values are stored as arbitrary-precision `NUMERIC` in PostgreSQL
and serialised as decimal strings (e.g. `"1234.56"`) in the API to avoid
floating-point rounding.

The frontend uses `Intl.NumberFormat` with the user’s locale and each
account’s currency code for display:

```ts
formatCurrency("1234.56", "EUR", "de-DE")  // → "1.234,56 €"
formatCurrency("1234.56", "USD", "en-US")  // → "$1,234.56"
```

See `web/src/lib/format.ts` for the implementation.

## ECharts locale

Chart axis labels and tooltip values use the same locale as the browser
(`navigator.language`). The `initChart` helper in `web/src/lib/chart.ts`
automatically loads the matching ECharts locale pack when available and falls
back to EN for unsupported locales.
