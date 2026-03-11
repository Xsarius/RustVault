-- Migration 0001: Initial Schema
--
-- Core entities: users, banks, accounts, categories, tags, sessions, audit_log, settings.
-- Establishes the foundational data model for RustVault Phase 1.
--
-- Design notes:
--   - UUIDs (v4) as primary keys via pgcrypto's gen_random_uuid().
--   - OIDC support: users have nullable password_hash (OIDC-only users have no password),
--     auth_provider enum, and a unique (oidc_issuer, oidc_subject) index.
--   - Hierarchical categories via self-referencing parent_id.
--   - Soft-archive pattern (is_archived boolean) instead of hard deletes.
--   - JSONB metadata columns for extensibility without schema migrations.
--   - Audit log tracks all create/update/delete mutations for compliance.
--   - Sessions table for server-side refresh token revocation.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================
-- Enum types
-- ============================================================

CREATE TYPE user_role AS ENUM ('admin', 'member', 'viewer');
CREATE TYPE auth_provider AS ENUM ('local', 'oidc', 'both');
CREATE TYPE account_type AS ENUM ('checking', 'savings', 'credit', 'investment', 'loan');
CREATE TYPE category_type AS ENUM ('expense', 'income');

-- ============================================================
-- Users
-- ============================================================

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      TEXT NOT NULL,
    email         TEXT NOT NULL,
    password_hash TEXT,
    role          user_role NOT NULL DEFAULT 'member',
    auth_provider auth_provider NOT NULL DEFAULT 'local',
    oidc_subject  TEXT,
    oidc_issuer   TEXT,
    locale        TEXT NOT NULL DEFAULT 'en-US',
    timezone      TEXT NOT NULL DEFAULT 'UTC',
    settings      JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_users_username UNIQUE (username),
    CONSTRAINT uq_users_email UNIQUE (email),
    CONSTRAINT chk_oidc_fields_together CHECK (
        (oidc_subject IS NULL) = (oidc_issuer IS NULL)
    )
);

CREATE UNIQUE INDEX idx_users_oidc ON users (oidc_issuer, oidc_subject)
    WHERE oidc_subject IS NOT NULL;

-- ============================================================
-- Banks
-- ============================================================

CREATE TABLE banks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    is_archived BOOLEAN NOT NULL DEFAULT false,
    sort_order  INT NOT NULL DEFAULT 0,
    metadata    JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_banks_user_name UNIQUE (user_id, name)
);

-- ============================================================
-- Accounts
-- ============================================================

CREATE TABLE accounts (
    id                         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    bank_id                    UUID NOT NULL REFERENCES banks (id) ON DELETE CASCADE,
    name                       TEXT NOT NULL,
    currency                   TEXT NOT NULL DEFAULT 'USD',
    type                       account_type NOT NULL DEFAULT 'checking',
    balance_cache              NUMERIC(19, 4) NOT NULL DEFAULT 0,
    supports_nonstandard_topup BOOLEAN NOT NULL DEFAULT false,
    is_archived                BOOLEAN NOT NULL DEFAULT false,
    sort_order                 INT NOT NULL DEFAULT 0,
    metadata                   JSONB NOT NULL DEFAULT '{}',
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_accounts_user_name UNIQUE (user_id, name)
);

-- ============================================================
-- Categories (hierarchical via parent_id)
-- ============================================================

CREATE TABLE categories (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    parent_id  UUID REFERENCES categories (id) ON DELETE SET NULL,
    icon       TEXT,
    color      TEXT,
    category_type category_type NOT NULL DEFAULT 'expense',
    sort_order INT NOT NULL DEFAULT 0,
    metadata   JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_categories_user_name_parent UNIQUE NULLS NOT DISTINCT (user_id, name, parent_id)
);

-- ============================================================
-- Tags
-- ============================================================

CREATE TABLE tags (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    color      TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_tags_user_name UNIQUE (user_id, name)
);

-- ============================================================
-- Sessions (refresh token tracking, server-side revocation)
-- ============================================================

CREATE TABLE sessions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL,
    user_agent    TEXT,
    ip_address    TEXT,
    expires_at    TIMESTAMPTZ NOT NULL,
    revoked       BOOLEAN NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- Audit Log
-- ============================================================

CREATE TABLE audit_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID REFERENCES users (id) ON DELETE SET NULL,
    entity_type TEXT NOT NULL,
    entity_id   UUID NOT NULL,
    action      TEXT NOT NULL,
    old_value   JSONB,
    new_value   JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- Triggers: auto-update updated_at
-- ============================================================

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_banks_updated_at
    BEFORE UPDATE ON banks
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_accounts_updated_at
    BEFORE UPDATE ON accounts
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ============================================================
-- Indexes
-- ============================================================

CREATE INDEX idx_banks_user_id ON banks (user_id);
CREATE INDEX idx_accounts_user_id ON accounts (user_id);
CREATE INDEX idx_accounts_bank_id ON accounts (bank_id);
CREATE INDEX idx_categories_user_id ON categories (user_id);
CREATE INDEX idx_categories_parent_id ON categories (parent_id);
CREATE INDEX idx_tags_user_id ON tags (user_id);
CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_expires ON sessions (expires_at) WHERE NOT revoked;
CREATE INDEX idx_audit_log_entity ON audit_log (entity_type, entity_id);
CREATE INDEX idx_audit_log_user_created ON audit_log (user_id, created_at DESC);
