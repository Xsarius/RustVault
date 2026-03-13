-- Migration 0004: Exchange Rates
--
-- Adds daily currency exchange rate storage for multi-currency budgeting.
--
-- Design notes:
--   - Rates are fetched daily from the ECB XML feed (primary) or Open Exchange
--     Rates (fallback). They are stored on a per-date basis so historical
--     reports can use the rate at the time of the transaction.
--   - base_currency is typically "EUR" (ECB) or "USD" (OXR) depending on source.
--   - A UNIQUE index on (base_currency, target_currency, date) prevents duplicate
--     rates and allows efficient upserts.
--   - BIGSERIAL id for efficient append-only writes from the rate fetcher task.

CREATE TABLE exchange_rates (
    id              BIGSERIAL PRIMARY KEY,
    base_currency   TEXT NOT NULL,
    target_currency TEXT NOT NULL,
    rate            NUMERIC(24, 10) NOT NULL,
    date            DATE NOT NULL,
    source          TEXT NOT NULL DEFAULT 'ecb',  -- 'ecb' | 'oxr' | 'manual'
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_exchange_rate UNIQUE (base_currency, target_currency, date)
);

CREATE INDEX idx_exchange_rates_lookup
    ON exchange_rates (base_currency, target_currency, date DESC);
