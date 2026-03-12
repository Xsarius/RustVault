-- Migration 0002: Transactions, Imports, Rules
--
-- Adds the core transaction engine tables for Phase 3A:
--   - transactions: all financial entries (income, expense, transfer).
--   - transaction_tags: junction table for many-to-many transaction ↔ tag.
--   - transfers: links two transactions as a transfer pair.
--   - imports: tracks file import sessions and their status.
--   - auto_rules: user-defined auto-categorization rules.
--
-- Design notes:
--   - Transactions have `transaction_type` enum (income/expense/transfer).
--   - Transfers link exactly two transactions with confidence scoring.
--   - Full-text search via tsvector + GIN index on transaction descriptions.
--   - Duplicate detection aided by (account_id, date, amount) index.
--   - Imports track the full lifecycle: pending → processing → completed/failed.
--   - Auto-rules use JSONB for flexible condition/action definitions.

-- ============================================================
-- Enum types
-- ============================================================

CREATE TYPE transaction_type AS ENUM ('income', 'expense', 'transfer');
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'rolled_back');
CREATE TYPE transfer_method AS ENUM ('internal', 'card_payment', 'wire', 'other');
CREATE TYPE transfer_status AS ENUM ('suggested', 'confirmed', 'rejected');

-- ============================================================
-- Imports (must exist before transactions for FK)
-- ============================================================

CREATE TABLE imports (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    file_name         TEXT NOT NULL,
    file_format       TEXT NOT NULL,
    account_id        UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    status            import_status NOT NULL DEFAULT 'pending',
    total_rows        INT NOT NULL DEFAULT 0,
    imported_count    INT NOT NULL DEFAULT 0,
    skipped_count     INT NOT NULL DEFAULT 0,
    duplicate_count   INT NOT NULL DEFAULT 0,
    error_count       INT NOT NULL DEFAULT 0,
    error_details     JSONB,
    column_mapping    JSONB,
    metadata          JSONB NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- Transactions
-- ============================================================

CREATE TABLE transactions (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    account_id        UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    category_id       UUID REFERENCES categories (id) ON DELETE SET NULL,
    import_id         UUID REFERENCES imports (id) ON DELETE SET NULL,
    transaction_type  transaction_type NOT NULL DEFAULT 'expense',
    amount            NUMERIC(19, 4) NOT NULL,
    currency          TEXT NOT NULL DEFAULT 'USD',
    date              DATE NOT NULL,
    description       TEXT NOT NULL DEFAULT '',
    original_desc     TEXT,
    payee             TEXT,
    reference         TEXT,
    notes             TEXT,
    is_reviewed       BOOLEAN NOT NULL DEFAULT false,
    is_deleted        BOOLEAN NOT NULL DEFAULT false,
    is_duplicate      BOOLEAN NOT NULL DEFAULT false,
    metadata          JSONB NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Full-text search vector (auto-maintained via trigger)
    search_vector     tsvector GENERATED ALWAYS AS (
        to_tsvector('simple',
            coalesce(description, '') || ' ' ||
            coalesce(original_desc, '') || ' ' ||
            coalesce(payee, '') || ' ' ||
            coalesce(notes, '')
        )
    ) STORED
);

-- ============================================================
-- Transaction Tags (junction table)
-- ============================================================

CREATE TABLE transaction_tags (
    transaction_id UUID NOT NULL REFERENCES transactions (id) ON DELETE CASCADE,
    tag_id         UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (transaction_id, tag_id)
);

-- ============================================================
-- Transfers
-- ============================================================

CREATE TABLE transfers (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    debit_tx_id       UUID NOT NULL REFERENCES transactions (id) ON DELETE CASCADE,
    credit_tx_id      UUID NOT NULL REFERENCES transactions (id) ON DELETE CASCADE,
    method            transfer_method NOT NULL DEFAULT 'internal',
    status            transfer_status NOT NULL DEFAULT 'confirmed',
    exchange_rate     NUMERIC(19, 8),
    confidence        NUMERIC(5, 2),
    notes             TEXT,
    metadata          JSONB NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_transfer_different_tx CHECK (debit_tx_id != credit_tx_id)
);

-- ============================================================
-- Auto-categorization Rules
-- ============================================================

CREATE TABLE auto_rules (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    priority    INT NOT NULL DEFAULT 0,
    is_enabled  BOOLEAN NOT NULL DEFAULT true,
    conditions  JSONB NOT NULL DEFAULT '[]',
    actions     JSONB NOT NULL DEFAULT '[]',
    metadata    JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- Triggers: auto-update updated_at
-- ============================================================

CREATE TRIGGER trg_transactions_updated_at
    BEFORE UPDATE ON transactions
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_imports_updated_at
    BEFORE UPDATE ON imports
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_auto_rules_updated_at
    BEFORE UPDATE ON auto_rules
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ============================================================
-- Indexes
-- ============================================================

-- Transaction lookup & filtering
CREATE INDEX idx_transactions_user_id ON transactions (user_id);
CREATE INDEX idx_transactions_account_id ON transactions (account_id);
CREATE INDEX idx_transactions_category_id ON transactions (category_id);
CREATE INDEX idx_transactions_import_id ON transactions (import_id);
CREATE INDEX idx_transactions_date ON transactions (user_id, date DESC);
CREATE INDEX idx_transactions_type ON transactions (user_id, transaction_type);
CREATE INDEX idx_transactions_reviewed ON transactions (user_id, is_reviewed) WHERE NOT is_deleted;

-- Duplicate detection: same account, date, amount
CREATE INDEX idx_transactions_dedup ON transactions (account_id, date, amount) WHERE NOT is_deleted;

-- Full-text search
CREATE INDEX idx_transactions_search ON transactions USING GIN (search_vector);

-- Soft-delete filter
CREATE INDEX idx_transactions_active ON transactions (user_id, date DESC) WHERE NOT is_deleted;

-- Transaction tags
CREATE INDEX idx_transaction_tags_tag ON transaction_tags (tag_id);

-- Transfers
CREATE INDEX idx_transfers_user_id ON transfers (user_id);
CREATE INDEX idx_transfers_debit_tx ON transfers (debit_tx_id);
CREATE INDEX idx_transfers_credit_tx ON transfers (credit_tx_id);

-- Imports
CREATE INDEX idx_imports_user_id ON imports (user_id);
CREATE INDEX idx_imports_account_id ON imports (account_id);

-- Auto-rules
CREATE INDEX idx_auto_rules_user_id ON auto_rules (user_id);
CREATE INDEX idx_auto_rules_priority ON auto_rules (user_id, priority);
