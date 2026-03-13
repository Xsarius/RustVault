-- Migration 0003: Budgets & Budget Lines
--
-- Adds the budgeting subsystem for Phase 4:
--   - budgets: a planned spending envelope for a time period.
--   - budget_lines: per-category planned limits within a budget.
--
-- Design notes:
--   - periods are open-ended: period_start + period_end define the window;
--     this supports monthly, quarterly, or any custom range.
--   - is_recurring + recurrence_rule (iCal RRULE subset) drive auto-generation
--     of next period shells.
--   - actual_amount_cache on budget_lines is a computed cache refreshed by the
--     budget service; it avoids expensive aggregations on every page load.
--   - currency holds the budget's reporting currency; amounts on budget_lines
--     are in this currency, converted from transaction currency at import time.

-- ============================================================
-- Budgets
-- ============================================================

CREATE TABLE budgets (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    period_start     DATE NOT NULL,
    period_end       DATE NOT NULL,
    currency         TEXT NOT NULL DEFAULT 'USD',
    is_recurring     BOOLEAN NOT NULL DEFAULT false,
    recurrence_rule  TEXT,               -- iCal RRULE string, e.g. "FREQ=MONTHLY"
    is_archived      BOOLEAN NOT NULL DEFAULT false,
    notes            TEXT,
    metadata         JSONB NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_budget_period CHECK (period_end >= period_start),
    CONSTRAINT chk_budget_recurrence CHECK (
        NOT is_recurring OR recurrence_rule IS NOT NULL
    )
);

-- ============================================================
-- Budget Lines (per-category planned amounts)
-- ============================================================

CREATE TABLE budget_lines (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id           UUID NOT NULL REFERENCES budgets (id) ON DELETE CASCADE,
    category_id         UUID REFERENCES categories (id) ON DELETE SET NULL,
    planned_amount      NUMERIC(19, 4) NOT NULL DEFAULT 0,
    actual_amount_cache NUMERIC(19, 4) NOT NULL DEFAULT 0,
    notes               TEXT,
    sort_order          INT NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- One line per category per budget (category_id can be NULL for "unallocated")
    CONSTRAINT uq_budget_line_category UNIQUE (budget_id, category_id)
);

-- ============================================================
-- Triggers: auto-update updated_at
-- ============================================================

CREATE TRIGGER trg_budgets_updated_at
    BEFORE UPDATE ON budgets
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_budget_lines_updated_at
    BEFORE UPDATE ON budget_lines
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ============================================================
-- Indexes
-- ============================================================

CREATE INDEX idx_budgets_user_id ON budgets (user_id);
CREATE INDEX idx_budgets_period ON budgets (user_id, period_start DESC, period_end DESC);
CREATE INDEX idx_budget_lines_budget_id ON budget_lines (budget_id);
CREATE INDEX idx_budget_lines_category_id ON budget_lines (category_id);
