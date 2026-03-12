# Multi-Currency

> **Status:** Planned for Phase 5. This feature is not yet implemented.

RustVault will support managing finances across multiple currencies with automatic exchange rate handling.

## Planned Features

- **Per-account currency** — each account has its own currency (e.g. EUR, USD, GBP)
- **Automatic exchange rates** — daily rates fetched from a public API
- **Base currency** — all reports and totals converted to your chosen base currency
- **Manual rate override** — set custom rates for specific transactions
- **Currency formatting** — locale-aware display (€1.000,00 vs $1,000.00)

## How It Will Work

Each account is assigned a currency at creation time. When viewing aggregate data (dashboard, reports), RustVault converts all amounts to your base currency using the exchange rate for the transaction date. Transfers between accounts with different currencies are recorded with the rate used.
