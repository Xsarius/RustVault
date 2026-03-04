# RustVault — API Implementation Plan

> Complete REST API specification for the RustVault backend.
> All endpoints are prefixed with `/api`. Authentication uses JWT Bearer tokens unless noted otherwise.

---

## Table of Contents

1. [API Conventions](#1-api-conventions)
2. [Authentication & Sessions](#2-authentication--sessions)
3. [Banks & Accounts](#3-banks--accounts)
4. [Categories](#4-categories)
5. [Tags](#5-tags)
6. [Transactions & Transfers](#6-transactions--transfers)
7. [Import Pipeline](#7-import-pipeline)
8. [Auto-Categorization Rules](#8-auto-categorization-rules)
9. [Budgets](#9-budgets)
10. [Reports & Analytics](#10-reports--analytics)
11. [Settings & i18n](#11-settings--i18n)
12. [AI Features](#12-ai-features)
13. [Admin](#13-admin)
14. [System](#14-system)
15. [WebSocket](#15-websocket)
16. [Implementation Phases](#16-implementation-phases)
17. [Endpoint Summary Table](#17-endpoint-summary-table)

---

## 1. API Conventions

### Base URL

```
/api
```

### Standard Response Envelope

**Success (single resource):**
```json
{
  "data": { ... }
}
```

**Success (collection / paginated):**
```json
{
  "data": [ ... ],
  "meta": {
    "total": 1250,
    "page_size": 50,
    "next_cursor": "eyJkYXRlIjoiMjAyNi0wMS0xNSIsImlkIjoiYWJjMTIzIn0=",
    "has_more": true
  }
}
```

**Error:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "One or more fields are invalid",
    "details": [
      { "field": "email", "message": "Invalid email format" }
    ]
  }
}
```

### Common Headers

**Request:**
| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes (protected) | `Bearer <access_token>` |
| `Content-Type` | Yes (POST/PUT) | `application/json` or `multipart/form-data` |
| `Accept-Language` | No | Locale preference (e.g., `en-US`, `pl-PL`). Affects error messages |
| `X-Requested-With` | Yes | `RustVault` — CSRF protection header |

**Response:**
| Header | Description |
|--------|-------------|
| `X-Request-Id` | UUID for request tracing |
| `ETag` | Resource version (on cacheable endpoints) |
| `X-RateLimit-Remaining` | Remaining requests in window |
| `X-RateLimit-Reset` | Seconds until rate limit resets |

### Pagination

Cursor-based pagination on all list endpoints:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | `50` | Items per page (max 100) |
| `cursor` | string | — | Opaque cursor from previous response's `meta.next_cursor` |

### Filtering & Sorting

List endpoints support query parameters for filtering. Naming convention:
- Filter by field: `?account_id=<uuid>&category_id=<uuid>`
- Date range: `?date_from=2026-01-01&date_to=2026-01-31`
- Search: `?q=search+text`
- Sort: `?sort=date&order=desc`

---

## 2. Authentication & Sessions

> **Rate limit:** 5 requests / 15 min / IP on login/register endpoints.

### 2.1 Register

```
POST /api/auth/register
```

**Auth:** None  
**Phase:** P1  

**Request Body:**
```json
{
  "username": "john",
  "email": "john@example.com",
  "password": "securepassword123"
}
```

| Field | Type | Validation |
|-------|------|------------|
| `username` | string | 2–50 chars, unique |
| `email` | string | Valid email, unique |
| `password` | string | 10–128 chars |

**Response `201`:**
```json
{
  "data": {
    "user": {
      "id": "uuid",
      "username": "john",
      "email": "john@example.com",
      "role": "admin",
      "auth_provider": "local",
      "locale": "en-US",
      "timezone": "UTC",
      "settings": {},
      "created_at": "2026-03-01T12:00:00Z"
    },
    "access_token": "eyJ...",
    "expires_in": 900
  }
}
```

**Set-Cookie:** `refresh_token=<opaque>; HttpOnly; Secure; SameSite=Strict; Path=/api; Max-Age=604800`

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `VALIDATION_ERROR` | Invalid input |
| `403` | `REGISTRATION_DISABLED` | `allow_new_user_register` is `false` in server config |
| `409` | `CONFLICT` | Email or username taken |
| `429` | `RATE_LIMITED` | Too many attempts |

---

### 2.2 Login

```
POST /api/auth/login
```

**Auth:** None  
**Phase:** P1  

**Request Body:**
```json
{
  "email": "john@example.com",
  "password": "securepassword123"
}
```

**Response `200`:**
```json
{
  "data": {
    "user": { ... },
    "access_token": "eyJ...",
    "expires_in": 900
  }
}
```

**Set-Cookie:** `refresh_token=<opaque>; HttpOnly; Secure; SameSite=Strict; Path=/api; Max-Age=604800`

**Errors:**
| Status | Code | When |
|--------|------|------|
| `401` | `INVALID_CREDENTIALS` | Wrong email or password |
| `403` | `ACCOUNT_LOCKED` | Too many failed attempts (20+) |
| `429` | `RATE_LIMITED` | Brute force protection |

---

### 2.3 Refresh Token

```
POST /api/auth/refresh
```

**Auth:** None (uses HttpOnly cookie)  
**Phase:** P1  

Uses the `refresh_token` from the HttpOnly cookie. Implements **rotation**: old refresh token is invalidated, a new one is issued.

**Response `200`:**
```json
{
  "data": {
    "access_token": "eyJ...",
    "expires_in": 900
  }
}
```

**Set-Cookie:** New `refresh_token` cookie (rotated).

**Errors:**
| Status | Code | When |
|--------|------|------|
| `401` | `TOKEN_EXPIRED` | Refresh token expired |
| `401` | `TOKEN_INVALID` | Token reuse detected (potential theft) — all sessions revoked |

---

### 2.4 Current User

```
GET /api/auth/me
```

**Auth:** Required  
**Phase:** P1  

**Response `200`:**
```json
{
  "data": {
    "id": "uuid",
    "username": "john",
    "email": "john@example.com",
    "role": "admin",
    "auth_provider": "local",
    "locale": "en-US",
    "timezone": "UTC",
    "settings": {
      "default_currency": "USD",
      "date_format": "YYYY-MM-DD",
      "ai_enabled": false,
      "theme": "dark"
    },
    "created_at": "2026-03-01T12:00:00Z"
  }
}
```

---

### 2.5 Logout

```
POST /api/auth/logout
```

**Auth:** Required  
**Phase:** P1  

Revokes the current session's refresh token.

**Response:** `204 No Content`

**Clear-Cookie:** `refresh_token` cleared.

---

### 2.6 Change Password

```
POST /api/auth/change-password
```

**Auth:** Required  
**Phase:** P1  

**Request Body:**
```json
{
  "current_password": "oldpassword123",
  "new_password": "newpassword456"
}
```

**Response:** `204 No Content`

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `VALIDATION_ERROR` | New password too short/long |
| `401` | `INVALID_CREDENTIALS` | Current password wrong |

---

### 2.7 List Sessions

```
GET /api/auth/sessions
```

**Auth:** Required  
**Phase:** P2  

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "ip_address": "192.168.1.1",
      "user_agent": "Mozilla/5.0...",
      "created_at": "2026-03-01T10:00:00Z",
      "last_used_at": "2026-03-02T08:30:00Z",
      "is_current": true
    }
  ]
}
```

---

### 2.8 Revoke Session

```
DELETE /api/auth/sessions/{session_id}
```

**Auth:** Required  
**Phase:** P2  

Revokes a specific session. Users can only revoke their own sessions.

**Response:** `204 No Content`

---

### 2.9 OIDC Configuration (Public)

```
GET /api/auth/oidc/config
```

**Auth:** None  
**Phase:** P1  

Returns OIDC configuration for the frontend to decide whether to show the SSO login button.

**Response `200`:**
```json
{
  "data": {
    "enabled": true,
    "display_name": "Authentik"
  }
}
```

If OIDC is not configured, returns `enabled: false`.

---

### 2.10 OIDC Authorize

```
GET /api/auth/oidc/authorize
```

**Auth:** None  
**Phase:** P1  

Initiates the OIDC Authorization Code flow with PKCE. Generates `state` + `code_verifier`, stores them server-side (or in an encrypted `HttpOnly` cookie), and returns a redirect to the OIDC provider's authorization endpoint.

**Response:** `302 Found` — redirect to OIDC provider (e.g., Authentik) authorization URL.

The redirect URL includes:
- `client_id`
- `redirect_uri` = `https://<host>/api/auth/oidc/callback`
- `response_type=code`
- `scope=openid profile email`
- `state=<csrf_token>`
- `code_challenge=<S256 hash of code_verifier>`
- `code_challenge_method=S256`

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `OIDC_NOT_CONFIGURED` | OIDC is not enabled on this instance |

---

### 2.11 OIDC Callback

```
GET /api/auth/oidc/callback
```

**Auth:** None  
**Phase:** P1  

Receives the authorization code from the OIDC provider after the user authenticates. This endpoint:

1. Validates the `state` parameter against the stored CSRF token.
2. Exchanges the authorization `code` for tokens via the provider's token endpoint (using the stored PKCE `code_verifier`).
3. Validates the `id_token` (signature via provider's JWKS, issuer, audience, expiry).
4. Extracts user claims: `sub`, `email`, `preferred_username`, `name`.
5. Provisions or links the user:
   - Look up by `(auth_provider, oidc_subject)` → existing OIDC user → login.
   - Look up by `email` → existing local user → link OIDC identity (set `oidc_subject`, `auth_provider='both'`).
   - No user found + `auto_register=true` → create user with `auth_provider='oidc'`, `password_hash=NULL`.
   - No user found + `auto_register=false` → return 403.
6. Issues local JWT access + refresh tokens (same as password login).
7. Redirects to frontend with session established.

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `code` | string | Authorization code from OIDC provider |
| `state` | string | CSRF state token for validation |

**Response:** `302 Found` — redirect to frontend at `/?oidc=success`.

**Set-Cookie:** `refresh_token=<opaque>; HttpOnly; Secure; SameSite=Strict; Path=/api; Max-Age=604800`

The access token is passed to the frontend via a short-lived, `HttpOnly` intermediary cookie or URL fragment, consumed once by the frontend's OIDC callback handler.

**Errors:**
| Status | Code | When |
|--------|------|------|
| `401` | `OIDC_AUTH_FAILED` | Code exchange or ID token validation failed |
| `401` | `OIDC_STATE_MISMATCH` | CSRF state does not match (possible CSRF attack) |
| `403` | `OIDC_USER_NOT_REGISTERED` | User exists in OIDC provider but `auto_register=false` and no local account |

---

### 2.12 Login (OIDC User Restriction)

When a user with `auth_provider='oidc'` (no local password) attempts `POST /api/auth/login`, the response is:

| Status | Code | When |
|--------|------|------|
| `400` | `PASSWORD_LOGIN_DISABLED` | Account uses SSO only — redirect to `/api/auth/oidc/authorize` |

Users with `auth_provider='both'` can use either login method.

---

## 3. Banks & Accounts

> **Hierarchy:** User → Banks → Accounts. A bank represents a financial institution (e.g., Revolut, Lunar, Zen).
> Each bank contains one or more accounts (checking, savings, credit card, etc.).
> Default grouping in UI is **by bank**. Filtering/grouping by account type is also supported.

### 3.1 List Banks

```
GET /api/banks
```

**Auth:** Required  
**Phase:** P1  

Returns all banks with their nested accounts.

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `include_archived` | bool | `false` | Include archived banks and accounts |

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "Revolut",
      "icon": "revolut",
      "color": "#0075EB",
      "country": "LT",
      "bic": "REVOLT21",
      "is_archived": false,
      "metadata": {},
      "created_at": "2026-01-15T12:00:00Z",
      "accounts": [
        {
          "id": "uuid",
          "name": "Main PLN",
          "currency": "PLN",
          "type": "checking",
          "balance_cache": "5234.50",
          "icon": "wallet",
          "color": "#3B82F6",
          "is_archived": false,
          "supports_card_topup": true,
          "metadata": {},
          "created_at": "2026-01-15T12:00:00Z"
        }
      ],
      "total_balance": {
        "amounts": [
          { "currency": "PLN", "amount": "5234.50" },
          { "currency": "EUR", "amount": "1200.00" }
        ]
      }
    }
  ]
}
```

---

### 3.2 Create Bank

```
POST /api/banks
```

**Auth:** Required  
**Phase:** P1  

**Request Body:**
```json
{
  "name": "Revolut",
  "icon": "revolut",
  "color": "#0075EB",
  "country": "LT",
  "bic": "REVOLT21",
  "metadata": {}
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `name` | string | Yes | 1–100 chars, unique per user |
| `icon` | string | No | Icon identifier |
| `color` | string | No | Hex color code |
| `country` | string | No | ISO 3166-1 alpha-2 (2 chars) |
| `bic` | string | No | BIC/SWIFT code (8 or 11 chars) |
| `metadata` | object | No | Max depth 5, max 64 KB |

**Response `201`:** Created bank object (without accounts — empty bank).

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `VALIDATION_ERROR` | Invalid input |
| `409` | `CONFLICT` | Duplicate bank name |

---

### 3.3 Update Bank

```
PUT /api/banks/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

**Request Body:** Partial update.
```json
{
  "name": "Revolut Business",
  "color": "#1A1A2E"
}
```

**Response `200`:** Updated bank object.

---

### 3.4 Delete (Archive) Bank

```
DELETE /api/banks/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

Soft-deletes by setting `is_archived = true`. All accounts within the bank are also archived. Transactions remain accessible.

**Response:** `204 No Content`

---

### 3.5 List Accounts

```
GET /api/accounts
```

**Auth:** Required  
**Phase:** P1  

Flat list of all accounts across all banks. Use `bank_id` filter to scope to one bank.

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `bank_id` | uuid | — | Filter by bank |
| `type` | string | — | Filter by account type |
| `include_archived` | bool | `false` | Include archived accounts |
| `group_by` | string | `bank` | Grouping: `bank`, `type`, `currency`, `none` |

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "bank_id": "uuid",
      "bank_name": "Revolut",
      "name": "Main PLN",
      "currency": "PLN",
      "type": "checking",
      "balance_cache": "5234.50",
      "icon": "wallet",
      "color": "#3B82F6",
      "is_archived": false,
      "supports_card_topup": true,
      "metadata": {},
      "created_at": "2026-01-15T12:00:00Z"
    }
  ]
}
```

---

### 3.6 Create Account

```
POST /api/accounts
```

**Auth:** Required  
**Phase:** P1  

**Request Body:**
```json
{
  "bank_id": "uuid",
  "name": "Main PLN",
  "currency": "PLN",
  "type": "checking",
  "icon": "wallet",
  "color": "#3B82F6",
  "supports_card_topup": false,
  "metadata": {}
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `bank_id` | uuid | Yes | Must exist, owned by user |
| `name` | string | Yes | 1–100 chars, unique per bank |
| `currency` | string | Yes | ISO 4217 code (3 chars) |
| `type` | enum | Yes | `checking`, `savings`, `cash`, `credit`, `investment`, `prepaid` |
| `icon` | string | No | Icon identifier |
| `color` | string | No | Hex color code |
| `supports_card_topup` | bool | No | Default `false`. Marks account as topup-able via card payment from other accounts |
| `metadata` | object | No | Max depth 5, max 64 KB |

**Response `201`:** Created account object.

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `VALIDATION_ERROR` | Invalid input |
| `409` | `CONFLICT` | Duplicate account name within bank |

---

### 3.7 Update Account

```
PUT /api/accounts/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

**Request Body:** Partial update — include only fields to change.
```json
{
  "name": "Updated Name",
  "icon": "wallet",
  "color": "#10B981",
  "supports_card_topup": true
}
```

Accounts can be moved between banks by updating `bank_id`.

**Response `200`:** Updated account object.

---

### 3.8 Delete (Archive) Account

```
DELETE /api/accounts/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

Soft-deletes by setting `is_archived = true`. Transactions remain accessible.

**Response:** `204 No Content`

---

## 4. Categories

### 4.1 List Categories

```
GET /api/categories
```

**Auth:** Required  
**Phase:** P1  

Returns hierarchical tree structure.

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `flat` | bool | `false` | Return flat list instead of tree |

**Response `200` (tree):**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "Food",
      "parent_id": null,
      "icon": "utensils",
      "color": "#EF4444",
      "is_income": false,
      "sort_order": 0,
      "metadata": {},
      "created_at": "2026-01-15T12:00:00Z",
      "children": [
        {
          "id": "uuid",
          "name": "Groceries",
          "parent_id": "uuid-of-food",
          "icon": "shopping-cart",
          "color": "#F87171",
          "is_income": false,
          "sort_order": 0,
          "metadata": {},
          "created_at": "2026-01-15T12:00:00Z",
          "children": []
        }
      ]
    }
  ]
}
```

---

### 4.2 Create Category

```
POST /api/categories
```

**Auth:** Required  
**Phase:** P1  

**Request Body:**
```json
{
  "name": "Groceries",
  "parent_id": "uuid-of-food",
  "icon": "shopping-cart",
  "color": "#F87171",
  "is_income": false,
  "sort_order": 0
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `name` | string | Yes | 1–100 chars, unique per user+parent |
| `parent_id` | uuid | No | Must exist, must belong to user |
| `icon` | string | No | Icon identifier |
| `color` | string | No | Hex color code |
| `is_income` | bool | No | Default `false` |
| `sort_order` | int | No | Default `0` |

**Response `201`:** Created category object.

---

### 4.3 Update Category

```
PUT /api/categories/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

Supports renaming, re-parenting, changing icon/color.

**Request Body:** Partial update.
```json
{
  "name": "Restaurants & Dining",
  "parent_id": "uuid-of-food",
  "color": "#F59E0B"
}
```

**Response `200`:** Updated category object.

---

### 4.4 Delete Category

```
DELETE /api/categories/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `merge_into` | uuid | — | Reassign transactions to this category. If omitted, transactions become uncategorized |

**Response:** `204 No Content`

---

### 4.5 Bulk Create Categories

```
POST /api/categories/bulk
```

**Auth:** Required  
**Phase:** P1  

Used during import to create multiple categories at once.

**Request Body:**
```json
{
  "categories": [
    { "name": "Subscriptions", "is_income": false },
    { "name": "Freelance", "is_income": true }
  ]
}
```

**Response `201`:**
```json
{
  "data": {
    "created": [ { ... }, { ... } ],
    "skipped": []
  }
}
```

`skipped` contains categories that already existed (returned with their existing IDs for linking).

---

## 5. Tags

### 5.1 List Tags

```
GET /api/tags
```

**Auth:** Required  
**Phase:** P1  

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "entertainment",
      "color": "#8B5CF6",
      "created_at": "2026-01-15T12:00:00Z"
    }
  ]
}
```

---

### 5.2 Create Tag

```
POST /api/tags
```

**Auth:** Required  
**Phase:** P1  

**Request Body:**
```json
{
  "name": "entertainment",
  "color": "#8B5CF6"
}
```

**Response `201`:** Created tag object.

---

### 5.3 Update Tag

```
PUT /api/tags/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

**Response `200`:** Updated tag object.

---

### 5.4 Delete Tag

```
DELETE /api/tags/{id}
```

**Auth:** Required (owner only)  
**Phase:** P1  

Removes the tag from all transactions where it was applied.

**Response:** `204 No Content`

---

### 5.5 Bulk Create Tags

```
POST /api/tags/bulk
```

**Auth:** Required  
**Phase:** P1  

**Request Body:**
```json
{
  "tags": [
    { "name": "recurring", "color": "#06B6D4" },
    { "name": "business", "color": "#14B8A6" }
  ]
}
```

**Response `201`:**
```json
{
  "data": {
    "created": [ ... ],
    "skipped": [ ... ]
  }
}
```

---

## 6. Transactions & Transfers

> **Transaction types:** Every transaction has a `type` field: `income`, `expense`, or `transfer`.
> Transfers between the user's own accounts (including cross-bank) are represented as a **linked pair** of transactions:
> one expense (debit) on the source account and one income (credit) on the destination account, connected by a `transfer_id`.
>
> **Internal transfer detection:** When importing bank statements, the system detects matching pairs:
> Bank A shows an outgoing transfer, Bank B shows an incoming transfer with matching amount/date.
> These are linked as a single internal transfer rather than counted as separate income/expense.
>
> **Card top-ups:** Transfers made via card payment (e.g., Revolut → Zen card top-up) are marked with
> `transfer_method: "card_payment"` to distinguish from wire/SEPA/internal transfers.

### 6.1 List Transactions

```
GET /api/transactions
```

**Auth:** Required  
**Phase:** P3  

Paginated, filterable, full-text searchable.

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `limit` | int | `50` | Items per page (max 100) |
| `cursor` | string | — | Pagination cursor |
| `account_id` | uuid | — | Filter by account |
| `bank_id` | uuid | — | Filter by bank (all accounts within that bank) |
| `category_id` | uuid | — | Filter by category |
| `tag_id` | uuid | — | Filter by tag |
| `type` | string | — | Filter: `income`, `expense`, `transfer` |
| `date_from` | date | — | Start date (inclusive) |
| `date_to` | date | — | End date (inclusive) |
| `amount_min` | decimal | — | Minimum absolute amount |
| `amount_max` | decimal | — | Maximum absolute amount |
| `is_reviewed` | bool | — | Filter by review status |
| `import_id` | uuid | — | Filter by import batch |
| `exclude_transfers` | bool | `false` | Exclude internal transfers (useful for income/expense reports) |
| `transfer_status` | string | — | Filter: `linked`, `unlinked`, `suggested` |
| `q` | string | — | Full-text search (description, payee, notes) |
| `sort` | string | `date` | Sort field: `date`, `amount`, `description` |
| `order` | string | `desc` | Sort order: `asc`, `desc` |

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "account_id": "uuid",
      "account_name": "Main PLN",
      "bank_id": "uuid",
      "bank_name": "Revolut",
      "date": "2026-01-15",
      "amount": "-42.50",
      "currency": "PLN",
      "type": "expense",
      "description": "Weekly groceries",
      "original_desc": "POS PURCHASE WHOLE FOODS MKT #10456",
      "category_id": "uuid",
      "category_name": "Groceries",
      "payee": "Whole Foods",
      "notes": null,
      "import_id": "uuid",
      "is_reviewed": true,
      "transfer": null,
      "tags": [
        { "id": "uuid", "name": "food", "color": "#EF4444" }
      ],
      "metadata": {},
      "created_at": "2026-01-15T12:00:00Z",
      "updated_at": "2026-01-16T09:30:00Z"
    },
    {
      "id": "uuid-debit-side",
      "account_id": "uuid-revolut-pln",
      "account_name": "Main PLN",
      "bank_id": "uuid-revolut",
      "bank_name": "Revolut",
      "date": "2026-01-20",
      "amount": "-500.00",
      "currency": "PLN",
      "type": "transfer",
      "description": "Transfer to Zen",
      "original_desc": "CARD PYMT ZEN.COM",
      "category_id": null,
      "category_name": null,
      "payee": null,
      "notes": null,
      "import_id": "uuid",
      "is_reviewed": true,
      "transfer": {
        "transfer_id": "uuid-transfer",
        "counterpart_transaction_id": "uuid-credit-side",
        "counterpart_account_id": "uuid-zen-pln",
        "counterpart_account_name": "Zen PLN",
        "counterpart_bank_name": "Zen",
        "direction": "outgoing",
        "method": "card_payment",
        "status": "linked"
      },
      "tags": [],
      "metadata": {},
      "created_at": "2026-01-20T12:00:00Z",
      "updated_at": "2026-01-20T12:00:00Z"
    }
  ],
  "meta": {
    "total": 1250,
    "page_size": 50,
    "next_cursor": "eyJkYXRlIjoiMjAyNi0wMS0xNSIsImlkIjoiYWJjIn0=",
    "has_more": true
  }
}
```

---

### 6.2 Get Transaction

```
GET /api/transactions/{id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

Returns full transaction with audit history and transfer details.

**Response `200`:**
```json
{
  "data": {
    "id": "uuid",
    "account_id": "uuid",
    "account_name": "Main PLN",
    "bank_id": "uuid",
    "bank_name": "Revolut",
    "date": "2026-01-20",
    "amount": "-500.00",
    "currency": "PLN",
    "type": "transfer",
    "description": "Transfer to Zen",
    "original_desc": "CARD PYMT ZEN.COM",
    "category_id": null,
    "payee": null,
    "notes": null,
    "import_id": "uuid",
    "is_reviewed": true,
    "transfer": {
      "transfer_id": "uuid-transfer",
      "counterpart_transaction_id": "uuid-credit-side",
      "counterpart_account_id": "uuid-zen-pln",
      "counterpart_account_name": "Zen PLN",
      "counterpart_bank_name": "Zen",
      "direction": "outgoing",
      "method": "card_payment",
      "status": "linked",
      "exchange_rate": null,
      "counterpart_amount": "500.00",
      "counterpart_currency": "PLN"
    },
    "tags": [],
    "metadata": {},
    "audit_history": [
      {
        "action": "transfer_linked",
        "changed_at": "2026-01-21T09:30:00Z",
        "changes": {
          "transfer_id": { "old": null, "new": "uuid-transfer" }
        }
      }
    ],
    "created_at": "2026-01-20T12:00:00Z",
    "updated_at": "2026-01-21T09:30:00Z"
  }
}
```

---

### 6.3 Create Transaction

```
POST /api/transactions
```

**Auth:** Required  
**Phase:** P3  

Manual transaction entry. For direct transfer creation, use `POST /api/transfers` instead.

**Request Body:**
```json
{
  "account_id": "uuid",
  "date": "2026-01-15",
  "amount": "-42.50",
  "currency": "PLN",
  "type": "expense",
  "description": "Weekly groceries",
  "category_id": "uuid",
  "payee": "Whole Foods",
  "notes": "Bought items for the week",
  "tag_ids": ["uuid1", "uuid2"],
  "metadata": {}
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `account_id` | uuid | Yes | Must exist, owned by user |
| `date` | date | Yes | ISO 8601 date |
| `amount` | decimal | Yes | Non-zero. Negative = expense/outgoing, positive = income/incoming |
| `currency` | string | No | ISO 4217. Defaults to account's currency |
| `type` | enum | Yes | `income`, `expense` (for transfers use the transfer endpoint) |
| `description` | string | Yes | 1–500 chars |
| `category_id` | uuid | No | Must exist, owned by user |
| `payee` | string | No | 0–200 chars |
| `notes` | string | No | 0–2000 chars |
| `tag_ids` | uuid[] | No | Each must exist, owned by user |
| `metadata` | object | No | Max depth 5, max 64 KB |

**Response `201`:** Created transaction object.

---

### 6.4 Update Transaction

```
PUT /api/transactions/{id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

All fields are editable. `original_desc` is immutable (not accepted in update).
If the transaction is part of a linked transfer, updating `amount` or `date` will prompt a warning but not update the counterpart automatically.

**Request Body:** Partial update — include only fields to change.
```json
{
  "description": "Weekly grocery shopping",
  "category_id": "uuid",
  "tag_ids": ["uuid1", "uuid3"],
  "is_reviewed": true
}
```

**Response `200`:** Updated transaction object.

---

### 6.5 Delete Transaction

```
DELETE /api/transactions/{id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

If the transaction is part of a linked transfer, only this side is deleted. The counterpart becomes an unlinked transaction (its `transfer` field is cleared).

**Response:** `204 No Content`

Account balance cache is refreshed automatically.

---

### 6.6 Bulk Update Transactions

```
PATCH /api/transactions/bulk
```

**Auth:** Required  
**Phase:** P3  

Apply the same change to multiple transactions at once.

**Request Body:**
```json
{
  "transaction_ids": ["uuid1", "uuid2", "uuid3"],
  "update": {
    "category_id": "uuid",
    "add_tags": ["uuid-tag-1"],
    "remove_tags": ["uuid-tag-2"],
    "is_reviewed": true,
    "payee": "Normalized Payee"
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `transaction_ids` | uuid[] | Yes | 1–500 IDs |
| `update.category_id` | uuid | No | Set category on all |
| `update.add_tags` | uuid[] | No | Add tags to all |
| `update.remove_tags` | uuid[] | No | Remove tags from all |
| `update.is_reviewed` | bool | No | Mark reviewed/unreviewed |
| `update.payee` | string | No | Set payee on all |

**Response `200`:**
```json
{
  "data": {
    "updated_count": 3
  }
}
```

---

### 6.7 Create Transfer

```
POST /api/transfers
```

**Auth:** Required  
**Phase:** P3  

Create a transfer between two of the user's accounts. Automatically creates two linked transactions (debit + credit).

**Request Body:**
```json
{
  "from_account_id": "uuid-revolut-pln",
  "to_account_id": "uuid-zen-pln",
  "date": "2026-01-20",
  "amount": "500.00",
  "from_currency": "PLN",
  "to_currency": "PLN",
  "exchange_rate": null,
  "method": "card_payment",
  "description": "Top up Zen via Revolut card",
  "notes": null,
  "metadata": {}
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `from_account_id` | uuid | Yes | Source account, owned by user |
| `to_account_id` | uuid | Yes | Destination account, owned by user, different from source |
| `date` | date | Yes | ISO 8601 date |
| `amount` | decimal | Yes | Positive number — the amount leaving the source account |
| `from_currency` | string | No | Defaults to source account's currency |
| `to_currency` | string | No | Defaults to destination account's currency |
| `exchange_rate` | decimal | No | Required if currencies differ. Rate: 1 `from_currency` = X `to_currency` |
| `method` | enum | No | `internal`, `wire`, `sepa`, `card_payment`, `other`. Default `internal` |
| `description` | string | No | 0–500 chars. Auto-generated if omitted |
| `notes` | string | No | 0–2000 chars |
| `metadata` | object | No | Max depth 5, max 64 KB |

**Response `201`:**
```json
{
  "data": {
    "transfer_id": "uuid",
    "from_transaction": {
      "id": "uuid-debit",
      "account_id": "uuid-revolut-pln",
      "account_name": "Main PLN",
      "bank_name": "Revolut",
      "amount": "-500.00",
      "currency": "PLN"
    },
    "to_transaction": {
      "id": "uuid-credit",
      "account_id": "uuid-zen-pln",
      "account_name": "Zen PLN",
      "bank_name": "Zen",
      "amount": "500.00",
      "currency": "PLN"
    },
    "method": "card_payment",
    "date": "2026-01-20"
  }
}
```

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `VALIDATION_ERROR` | Invalid input |
| `400` | `SAME_ACCOUNT` | Source and destination are the same account |
| `400` | `EXCHANGE_RATE_REQUIRED` | Currencies differ but no exchange rate provided |

---

### 6.8 Link Transactions as Transfer

```
POST /api/transfers/link
```

**Auth:** Required  
**Phase:** P3  

Manually link two existing transactions as a transfer pair (e.g., when auto-detection missed a match).

**Request Body:**
```json
{
  "debit_transaction_id": "uuid-outgoing",
  "credit_transaction_id": "uuid-incoming",
  "method": "wire"
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `debit_transaction_id` | uuid | Yes | Must be a negative-amount transaction, owned by user |
| `credit_transaction_id` | uuid | Yes | Must be a positive-amount transaction, owned by user, different account |
| `method` | enum | No | `internal`, `wire`, `sepa`, `card_payment`, `other`. Default `internal` |

**Response `200`:**
```json
{
  "data": {
    "transfer_id": "uuid",
    "debit_transaction_id": "uuid-outgoing",
    "credit_transaction_id": "uuid-incoming",
    "method": "wire",
    "status": "linked"
  }
}
```

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `ALREADY_LINKED` | One or both transactions are already part of a transfer |
| `400` | `SAME_ACCOUNT` | Both transactions are on the same account |
| `400` | `INVALID_DIRECTION` | Debit is not negative or credit is not positive |

---

### 6.9 Unlink Transfer

```
DELETE /api/transfers/{transfer_id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

Unlinks two transactions from a transfer. Both transactions remain as standalone income/expense. Does NOT delete the transactions themselves.

**Response:** `204 No Content`

---

### 6.10 Detect Transfers

```
POST /api/transfers/detect
```

**Auth:** Required  
**Phase:** P3  

Scan unlinked transactions across the user's accounts and suggest potential transfer pairs.

**Request Body:**
```json
{
  "date_from": "2026-01-01",
  "date_to": "2026-01-31",
  "account_ids": ["uuid1", "uuid2"],
  "date_tolerance_days": 3,
  "amount_tolerance_percent": 1.0,
  "auto_link": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `date_from` | date | — | Required. Start of scan range |
| `date_to` | date | — | Required. End of scan range |
| `account_ids` | uuid[] | All | Scope to specific accounts |
| `date_tolerance_days` | int | `3` | Max days apart for a match (banks process at different speeds) |
| `amount_tolerance_percent` | decimal | `1.0` | Allow small differences (fees, rounding). 0 = exact match only |
| `auto_link` | bool | `false` | If `true`, automatically link high-confidence matches (>= 95%). If `false`, return suggestions only |

**Response `200`:**
```json
{
  "data": {
    "suggestions": [
      {
        "confidence": 0.97,
        "debit_transaction": {
          "id": "uuid",
          "account_name": "Revolut PLN",
          "bank_name": "Revolut",
          "date": "2026-01-20",
          "amount": "-500.00",
          "description": "CARD PYMT ZEN.COM"
        },
        "credit_transaction": {
          "id": "uuid",
          "account_name": "Zen PLN",
          "bank_name": "Zen",
          "date": "2026-01-20",
          "amount": "500.00",
          "description": "CARD TOPUP REVOLUT"
        },
        "suggested_method": "card_payment",
        "amount_difference": "0.00",
        "date_difference_days": 0
      }
    ],
    "auto_linked_count": 0,
    "total_candidates_scanned": 450
  }
}
```

---

## 7. Import Pipeline

> **Rate limit:** 10 imports / user / hour. Max file size: 50 MB.
>
> **Transfer detection:** After transactions are imported, the pipeline automatically scans
> for matching transactions across the user's other accounts (same amount, close date)
> and suggests or auto-links them as internal transfers. This prevents double-counting
> a transfer as both an expense and an income in reports.

### 7.1 Upload File

```
POST /api/import/upload
```

**Auth:** Required  
**Phase:** P3  
**Content-Type:** `multipart/form-data`

Upload a bank statement file. The server auto-detects the format and returns a preview.

**Request:**
- `file`: binary file (CSV, MT940, OFX, QFX, QIF, CAMT.053 XML, XLSX, XLS, ODS, JSON, PDF)
- `account_id`: uuid (optional — can be set in configure step)

**Response `200`:**
```json
{
  "data": {
    "upload_id": "uuid",
    "detected_format": "csv",
    "preview_rows": [
      {
        "date": "2026-01-15",
        "amount": "-42.50",
        "description": "WHOLE FOODS MKT",
        "raw_data": { "col1": "01/15/2026", "col2": "-42.50", "col3": "WHOLE FOODS MKT" }
      }
    ],
    "total_rows": 156,
    "requires_mapping": true,
    "detected_columns": ["Date", "Amount", "Description", "Reference"],
    "saved_mapping": null
  }
}
```

If a saved column mapping exists for this account + format combination, `saved_mapping` contains it and `requires_mapping` is `false`.

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `UNSUPPORTED_FORMAT` | Unrecognized file type |
| `400` | `INVALID_MIME_TYPE` | Content doesn't match extension |
| `413` | `FILE_TOO_LARGE` | Exceeds max file size |
| `429` | `RATE_LIMITED` | Import rate limit exceeded |

---

### 7.2 Configure Import

```
POST /api/import/configure
```

**Auth:** Required  
**Phase:** P3  

Submit column mapping and target account for formats that require configuration (CSV, XLSX, JSON).

**Request Body:**
```json
{
  "upload_id": "uuid",
  "account_id": "uuid",
  "column_mapping": {
    "date": "col1",
    "amount": "col2",
    "description": "col3",
    "payee": null,
    "currency": null,
    "reference": "col4",
    "category": null,
    "notes": null
  },
  "date_format": "DD/MM/YYYY",
  "decimal_separator": ".",
  "save_mapping": true,
  "sheet_index": 0
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `upload_id` | uuid | Yes | From upload step |
| `account_id` | uuid | Yes | Target account |
| `column_mapping` | object | Yes (CSV/XLSX/JSON) | Maps source columns to fields |
| `date_format` | string | No | Override auto-detected date format |
| `decimal_separator` | string | No | `.` or `,` |
| `save_mapping` | bool | No | Save this mapping for future imports to same account |
| `sheet_index` | int | No | Which sheet to use (XLSX only, default `0`) |

**Response `200`:**
```json
{
  "data": {
    "upload_id": "uuid",
    "parsed_count": 156,
    "preview_rows": [ ... ],
    "warnings": [
      { "row": 45, "message": "Date could not be parsed, used fallback format" }
    ]
  }
}
```

---

### 7.3 Execute Import

```
POST /api/import/execute
```

**Auth:** Required  
**Phase:** P3  

Run the full import pipeline: parse → deduplicate → auto-categorize → [AI enrichment] → **detect transfers** → persist.

**Request Body:**
```json
{
  "upload_id": "uuid",
  "skip_duplicates": true,
  "apply_rules": true,
  "ai_enrich": false,
  "mark_as_reviewed": false,
  "detect_transfers": true,
  "auto_link_transfers": false,
  "transfer_date_tolerance_days": 3,
  "transfer_amount_tolerance_percent": 1.0
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `upload_id` | uuid | — | Required |
| `skip_duplicates` | bool | `true` | Skip detected duplicates |
| `apply_rules` | bool | `true` | Run auto-categorization rules |
| `ai_enrich` | bool | `false` | Run AI categorization + payee normalization (requires AI enabled) |
| `mark_as_reviewed` | bool | `false` | Mark all imported transactions as reviewed |
| `detect_transfers` | bool | `true` | Scan imported transactions against existing ones for transfer matches |
| `auto_link_transfers` | bool | `false` | Automatically link high-confidence transfer matches (>= 95%). When `false`, matches are returned as suggestions only |
| `transfer_date_tolerance_days` | int | `3` | Max days between matched transactions for transfer detection |
| `transfer_amount_tolerance_percent` | decimal | `1.0` | Allow small amount differences (fees, rounding). 0 = exact match only |

**Response `200`:**
```json
{
  "data": {
    "import_id": "uuid",
    "status": "completed",
    "total_rows": 156,
    "imported_rows": 148,
    "skipped_rows": 5,
    "duplicate_rows": 3,
    "error_rows": 0,
    "new_categories_created": [
      { "id": "uuid", "name": "Auto-created Category" }
    ],
    "new_tags_created": [],
    "transfers": {
      "auto_linked": 2,
      "suggestions": [
        {
          "confidence": 0.92,
          "imported_transaction_id": "uuid",
          "imported_description": "TRANSFER TO ZEN",
          "imported_amount": "-500.00",
          "match_transaction_id": "uuid",
          "match_account_name": "Zen PLN",
          "match_bank_name": "Zen",
          "match_description": "TOPUP FROM REVOLUT",
          "match_amount": "500.00",
          "suggested_method": "card_payment",
          "date_difference_days": 0,
          "amount_difference": "0.00"
        }
      ]
    },
    "errors": [],
    "warnings": [
      { "row": 12, "message": "Amount parsed as 0 — skipped" }
    ]
  }
}
```

---

### 7.4 List Imports

```
GET /api/imports
```

**Auth:** Required  
**Phase:** P3  

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `limit` | int | `20` | Items per page |
| `cursor` | string | — | Pagination cursor |
| `account_id` | uuid | — | Filter by account |
| `status` | string | — | Filter: `pending`, `processing`, `completed`, `failed`, `rolled_back` |

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "account_id": "uuid",
      "filename": "bank_statement_jan_2026.csv",
      "format": "csv",
      "status": "completed",
      "total_rows": 156,
      "imported_rows": 148,
      "skipped_rows": 5,
      "error_rows": 0,
      "created_at": "2026-01-31T14:00:00Z"
    }
  ],
  "meta": { ... }
}
```

---

### 7.5 Get Import Details

```
GET /api/imports/{id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

**Response `200`:**
```json
{
  "data": {
    "id": "uuid",
    "account_id": "uuid",
    "account_name": "Main PLN",
    "bank_id": "uuid",
    "bank_name": "Revolut",
    "filename": "bank_statement_jan_2026.csv",
    "format": "csv",
    "status": "completed",
    "total_rows": 156,
    "imported_rows": 148,
    "skipped_rows": 5,
    "error_rows": 0,
    "transfers_auto_linked": 2,
    "transfers_suggested": 1,
    "column_mapping": { ... },
    "errors": [],
    "warnings": [],
    "created_at": "2026-01-31T14:00:00Z",
    "transactions_preview": [
      { "id": "uuid", "date": "2026-01-15", "amount": "-42.50", "description": "...", "type": "expense", "transfer": null }
    ]
  }
}
```

---

### 7.6 Rollback Import

```
DELETE /api/imports/{id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

Deletes all transactions created by this import and updates the import status to `rolled_back`. Account balances are refreshed.

**Response:** `204 No Content`

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `BAD_REQUEST` | Import was already rolled back |
| `404` | `NOT_FOUND` | Import doesn't exist or not owned |

---

## 8. Auto-Categorization Rules

### 8.1 List Rules

```
GET /api/rules
```

**Auth:** Required  
**Phase:** P3  

Returns rules sorted by priority (ascending — lower number = higher priority).

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "Spotify Subscription",
      "priority": 1,
      "conditions": {
        "type": "and",
        "conditions": [
          { "type": "description_contains", "value": "SPOTIFY", "case_sensitive": false },
          { "type": "amount_range", "min": "-15.00", "max": "-5.00" }
        ]
      },
      "actions": [
        { "type": "set_category", "category_id": "uuid" },
        { "type": "add_tag", "tag_id": "uuid" },
        { "type": "set_payee", "payee": "Spotify" }
      ],
      "is_active": true,
      "match_count": 24,
      "created_at": "2026-01-10T12:00:00Z"
    }
  ]
}
```

---

### 8.2 Create Rule

```
POST /api/rules
```

**Auth:** Required  
**Phase:** P3  

**Request Body:**
```json
{
  "name": "Spotify Subscription",
  "priority": 1,
  "conditions": {
    "type": "or",
    "conditions": [
      { "type": "description_contains", "value": "SPOTIFY", "case_sensitive": false },
      { "type": "payee_contains", "value": "spotify" }
    ]
  },
  "actions": [
    { "type": "set_category", "category_id": "uuid" },
    { "type": "set_payee", "payee": "Spotify" }
  ],
  "is_active": true
}
```

**Condition Types:**
| Type | Fields | Description |
|------|--------|-------------|
| `and` | `conditions: []` | All must match |
| `or` | `conditions: []` | Any must match |
| `description_contains` | `value`, `case_sensitive` | Description substring match |
| `description_regex` | `pattern` | Regex match on description |
| `payee_equals` | `value` | Exact payee match |
| `payee_contains` | `value` | Payee substring match |
| `amount_range` | `min?`, `max?` | Amount within range |
| `account_id` | `value` | Specific account |

**Action Types:**
| Type | Fields | Description |
|------|--------|-------------|
| `set_category` | `category_id` | Set transaction category |
| `add_tag` | `tag_id` | Add tag |
| `set_payee` | `payee` | Normalize payee name |
| `set_metadata` | `key`, `value` | Set metadata field |

**Response `201`:** Created rule object.

---

### 8.3 Update Rule

```
PUT /api/rules/{id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

**Response `200`:** Updated rule object.

---

### 8.4 Delete Rule

```
DELETE /api/rules/{id}
```

**Auth:** Required (owner only)  
**Phase:** P3  

**Response:** `204 No Content`

---

### 8.5 Test Rule

```
POST /api/rules/test
```

**Auth:** Required  
**Phase:** P3  

Test a rule definition against existing transactions without saving or applying it.

**Request Body:**
```json
{
  "conditions": {
    "type": "description_contains",
    "value": "SPOTIFY",
    "case_sensitive": false
  },
  "account_id": "uuid",
  "limit": 20
}
```

**Response `200`:**
```json
{
  "data": {
    "match_count": 24,
    "preview_matches": [
      {
        "id": "uuid",
        "date": "2026-01-15",
        "amount": "-9.99",
        "description": "SPOTIFY AB STOCKHOLM SE"
      }
    ]
  }
}
```

---

### 8.6 Re-run Rules

```
POST /api/rules/rerun
```

**Auth:** Required  
**Phase:** P3  

Re-evaluate all active rules against existing transactions. Returns a preview of changes before applying.

**Request Body:**
```json
{
  "account_id": "uuid",
  "date_from": "2026-01-01",
  "date_to": "2026-01-31",
  "dry_run": true
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `account_id` | uuid | — | Optional, scope to one account |
| `date_from` | date | — | Optional, scope by date |
| `date_to` | date | — | Optional, scope by date |
| `dry_run` | bool | `true` | If `true`, return preview; if `false`, apply changes |

**Response `200`:**
```json
{
  "data": {
    "total_evaluated": 500,
    "total_matched": 45,
    "changes": [
      {
        "transaction_id": "uuid",
        "rule_id": "uuid",
        "rule_name": "Spotify Subscription",
        "actions_applied": [
          { "type": "set_category", "category_id": "uuid", "category_name": "Subscriptions" }
        ]
      }
    ]
  }
}
```

---

## 9. Budgets

### 9.1 List Budgets

```
GET /api/budgets
```

**Auth:** Required  
**Phase:** P4  

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `date_from` | date | — | Filter budgets overlapping start date |
| `date_to` | date | — | Filter budgets overlapping end date |
| `is_recurring` | bool | — | Filter by recurrence status |

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "name": "March 2026",
      "period_start": "2026-03-01",
      "period_end": "2026-03-31",
      "currency": "USD",
      "is_recurring": true,
      "recurrence_rule": "monthly",
      "total_planned": "3500.00",
      "total_actual": "2150.75",
      "created_at": "2026-02-28T12:00:00Z"
    }
  ]
}
```

---

### 9.2 Create Budget

```
POST /api/budgets
```

**Auth:** Required  
**Phase:** P4  

**Request Body:**
```json
{
  "name": "March 2026",
  "period_start": "2026-03-01",
  "period_end": "2026-03-31",
  "currency": "USD",
  "is_recurring": true,
  "recurrence_rule": "monthly"
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `name` | string | Yes | 1–100 chars |
| `period_start` | date | Yes | ISO 8601 |
| `period_end` | date | Yes | Must be after `period_start` |
| `currency` | string | Yes | ISO 4217 |
| `is_recurring` | bool | No | Default `false` |
| `recurrence_rule` | string | No | `monthly`, `quarterly`, `yearly` |

**Response `201`:** Created budget object.

---

### 9.3 Get Budget (with lines)

```
GET /api/budgets/{id}
```

**Auth:** Required (owner only)  
**Phase:** P4  

Returns budget with all budget lines and actual amounts computed.

**Response `200`:**
```json
{
  "data": {
    "id": "uuid",
    "name": "March 2026",
    "period_start": "2026-03-01",
    "period_end": "2026-03-31",
    "currency": "USD",
    "is_recurring": true,
    "recurrence_rule": "monthly",
    "lines": [
      {
        "id": "uuid",
        "category_id": "uuid",
        "category_name": "Groceries",
        "planned_amount": "500.00",
        "actual_amount": "387.25",
        "remaining": "112.75",
        "percent_used": 77.45,
        "notes": null
      }
    ],
    "summary": {
      "total_planned_income": "5000.00",
      "total_actual_income": "5000.00",
      "total_planned_expenses": "3500.00",
      "total_actual_expenses": "2150.75",
      "net_planned": "1500.00",
      "net_actual": "2849.25",
      "over_budget_categories": ["uuid-entertainment"]
    },
    "created_at": "2026-02-28T12:00:00Z"
  }
}
```

---

### 9.4 Update Budget

```
PUT /api/budgets/{id}
```

**Auth:** Required (owner only)  
**Phase:** P4  

Update budget metadata (name, period, recurrence).

**Response `200`:** Updated budget object.

---

### 9.5 Delete Budget

```
DELETE /api/budgets/{id}
```

**Auth:** Required (owner only)  
**Phase:** P4  

**Response:** `204 No Content`

---

### 9.6 Add Budget Line

```
POST /api/budgets/{id}/lines
```

**Auth:** Required (owner only)  
**Phase:** P4  

**Request Body:**
```json
{
  "category_id": "uuid",
  "planned_amount": "500.00",
  "notes": "Weekly groceries budget"
}
```

**Response `201`:** Created budget line object.

---

### 9.7 Update Budget Line

```
PUT /api/budgets/{id}/lines/{line_id}
```

**Auth:** Required (owner only)  
**Phase:** P4  

**Request Body:**
```json
{
  "planned_amount": "600.00",
  "notes": "Updated after holiday season"
}
```

**Response `200`:** Updated budget line object.

---

### 9.8 Delete Budget Line

```
DELETE /api/budgets/{id}/lines/{line_id}
```

**Auth:** Required (owner only)  
**Phase:** P4  

**Response:** `204 No Content`

---

### 9.9 Bulk Set Budget Lines

```
POST /api/budgets/{id}/lines/bulk
```

**Auth:** Required (owner only)  
**Phase:** P4  

Set multiple budget lines at once (upsert: creates or updates).

**Request Body:**
```json
{
  "lines": [
    { "category_id": "uuid", "planned_amount": "500.00" },
    { "category_id": "uuid", "planned_amount": "200.00", "notes": "dining out" }
  ]
}
```

**Response `200`:**
```json
{
  "data": {
    "created": 2,
    "updated": 0
  }
}
```

---

### 9.10 Budget Summary

```
GET /api/budgets/{id}/summary
```

**Auth:** Required (owner only)  
**Phase:** P4  

Returns detailed computed summary with variances.

**Response `200`:**
```json
{
  "data": {
    "total_planned_income": "5000.00",
    "total_actual_income": "5000.00",
    "total_planned_expenses": "3500.00",
    "total_actual_expenses": "2150.75",
    "net_planned": "1500.00",
    "net_actual": "2849.25",
    "by_category": [
      {
        "category_id": "uuid",
        "category_name": "Groceries",
        "planned": "500.00",
        "actual": "387.25",
        "variance": "112.75",
        "percent_used": 77.45
      }
    ],
    "over_budget_categories": [
      {
        "category_id": "uuid",
        "category_name": "Entertainment",
        "planned": "100.00",
        "actual": "145.00",
        "over_by": "45.00"
      }
    ],
    "daily_cumulative": [
      { "date": "2026-03-01", "actual": "45.00", "even_pace": "112.90" },
      { "date": "2026-03-02", "actual": "120.00", "even_pace": "225.80" }
    ]
  }
}
```

---

### 9.11 Copy Budget

```
POST /api/budgets/{id}/copy
```

**Auth:** Required (owner only)  
**Phase:** P4  

Duplicate planned amounts to a new period.

**Request Body:**
```json
{
  "name": "April 2026",
  "period_start": "2026-04-01",
  "period_end": "2026-04-30"
}
```

**Response `201`:** New budget object with copied lines (actual amounts reset to 0).

---

## 10. Reports & Analytics

> **Rate limit:** 20 requests / min / user on report endpoints.

### 10.1 Dashboard Summary

```
GET /api/reports/summary
```

**Auth:** Required  
**Phase:** P5  

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `currency` | string | User's default | Reporting currency (convert all amounts) |
| `account_ids` | uuid[] | All | Comma-separated account IDs to include |

**Response `200`:**
```json
{
  "data": {
    "net_worth": "45230.50",
    "total_income_this_month": "5000.00",
    "total_expenses_this_month": "2150.75",
    "savings_rate_this_month": 0.569,
    "income_trend": 0.02,
    "expense_trend": -0.05,
    "unreviewed_count": 12,
    "accounts": [
      {
        "id": "uuid",
        "name": "Main Checking",
        "balance": "5234.50",
        "currency": "USD"
      }
    ]
  }
}
```

---

### 10.2 Income vs Expense

```
GET /api/reports/income-expense
```

**Auth:** Required  
**Phase:** P5  

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `date_from` | date | 12 months ago | Start date |
| `date_to` | date | Today | End date |
| `granularity` | string | `monthly` | `daily`, `weekly`, `monthly`, `quarterly`, `yearly` |
| `currency` | string | User's default | Reporting currency |
| `account_ids` | uuid[] | All | Filter by accounts |

**Response `200`:**
```json
{
  "data": {
    "periods": [
      {
        "period": "2026-01",
        "income": "5000.00",
        "expenses": "3200.00",
        "net": "1800.00",
        "income_by_category": [
          { "category_id": "uuid", "name": "Salary", "amount": "4500.00" },
          { "category_id": "uuid", "name": "Freelance", "amount": "500.00" }
        ],
        "expenses_by_category": [
          { "category_id": "uuid", "name": "Groceries", "amount": "450.00" },
          { "category_id": "uuid", "name": "Rent", "amount": "1200.00" }
        ]
      }
    ],
    "totals": {
      "income": "60000.00",
      "expenses": "38400.00",
      "net": "21600.00",
      "average_savings_rate": 0.36
    }
  }
}
```

---

### 10.3 Category Trend

```
GET /api/reports/category/{id}/trend
```

**Auth:** Required  
**Phase:** P5  

Spending/income trend for a specific category over time.

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `date_from` | date | 12 months ago | Start date |
| `date_to` | date | Today | End date |
| `granularity` | string | `monthly` | Time granularity |
| `include_children` | bool | `true` | Include subcategory amounts |

**Response `200`:**
```json
{
  "data": {
    "category_id": "uuid",
    "category_name": "Food",
    "periods": [
      { "period": "2026-01", "amount": "650.00", "transaction_count": 28 },
      { "period": "2026-02", "amount": "580.00", "transaction_count": 24 }
    ],
    "average": "615.00",
    "min": "480.00",
    "max": "780.00",
    "top_payees": [
      { "payee": "Whole Foods", "total": "1200.00", "count": 12 },
      { "payee": "Trader Joe's", "total": "800.00", "count": 8 }
    ],
    "anomaly_months": [
      { "period": "2025-12", "amount": "780.00", "deviation_from_avg": "+26.8%" }
    ]
  }
}
```

---

### 10.4 Balance History

```
GET /api/reports/balance-history
```

**Auth:** Required  
**Phase:** P5  

Account balance over time, reconstructed from transactions.

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `date_from` | date | 12 months ago | Start date |
| `date_to` | date | Today | End date |
| `account_ids` | uuid[] | All | Accounts to include |
| `granularity` | string | `daily` | `daily`, `weekly`, `monthly` |
| `currency` | string | User's default | Convert to reporting currency |

**Response `200`:**
```json
{
  "data": {
    "series": [
      {
        "account_id": "uuid",
        "account_name": "Main Checking",
        "points": [
          { "date": "2026-01-01", "balance": "4500.00" },
          { "date": "2026-01-02", "balance": "4457.50" }
        ]
      }
    ],
    "net_worth_series": [
      { "date": "2026-01-01", "balance": "42000.00" },
      { "date": "2026-01-02", "balance": "41957.50" }
    ]
  }
}
```

Note: For time series > 1000 data points, the server applies LTTB downsampling automatically.

---

### 10.5 Cash Flow Analysis

```
GET /api/reports/cash-flow
```

**Auth:** Required  
**Phase:** P5  

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `date_from` | date | 6 months ago | Start date |
| `date_to` | date | Today | End date |
| `forecast_months` | int | `3` | Months to project (0-12) |
| `currency` | string | User's default | Reporting currency |

**Response `200`:**
```json
{
  "data": {
    "historical": [
      {
        "period": "2026-01",
        "inflow": "5000.00",
        "outflow": "3200.00",
        "net_flow": "1800.00",
        "cumulative": "1800.00"
      }
    ],
    "forecast": [
      {
        "period": "2026-04",
        "projected_inflow": "5100.00",
        "projected_outflow": "3300.00",
        "projected_net": "1800.00",
        "confidence": 0.85
      }
    ],
    "rolling_averages": {
      "avg_30d_income": "5000.00",
      "avg_30d_expenses": "3200.00",
      "avg_90d_income": "4900.00",
      "avg_90d_expenses": "3150.00"
    }
  }
}
```

---

### 10.6 Export Transactions

```
GET /api/export/transactions
```

**Auth:** Required  
**Phase:** P5  

Export transaction data in various formats for external use or backup.

**Query Parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `format` | string | `csv` | Export format: `csv`, `json`, `qif` |
| `date_from` | date | — | Start date (inclusive) |
| `date_to` | date | — | End date (inclusive) |
| `account_ids` | uuid[] | All | Filter by accounts |
| `category_ids` | uuid[] | All | Filter by categories |
| `include_transfers` | bool | `true` | Include transfer transactions |
| `currency` | string | Original | Convert amounts to specified currency (uses exchange rates at transaction date) |

**Response `200` (CSV):**

**Headers:**
```
Content-Type: text/csv; charset=utf-8
Content-Disposition: attachment; filename="rustvault-export-2026-03-04.csv"
```

**Body:**
```csv
date,description,amount,currency,category,account,tags,type,notes
2026-01-15,"Whole Foods Market",-42.50,USD,Groceries,Main Checking,"food,weekly",expense,""
2026-01-15,"Salary",5000.00,USD,Salary,Main Checking,"",income,""
```

**Response `200` (JSON):**

**Headers:**
```
Content-Type: application/json
Content-Disposition: attachment; filename="rustvault-export-2026-03-04.json"
```

**Body:**
```json
{
  "data": {
    "exported_at": "2026-03-04T12:00:00Z",
    "transaction_count": 156,
    "date_range": { "from": "2026-01-01", "to": "2026-03-04" },
    "transactions": [
      {
        "date": "2026-01-15",
        "description": "Whole Foods Market",
        "amount": "-42.50",
        "currency": "USD",
        "category": "Groceries",
        "account": "Main Checking",
        "tags": ["food", "weekly"],
        "type": "expense",
        "notes": null
      }
    ]
  }
}
```

**Response `200` (QIF):**

**Headers:**
```
Content-Type: application/qif
Content-Disposition: attachment; filename="rustvault-export-2026-03-04.qif"
```

**Body:**
```
!Type:Bank
D01/15/2026
T-42.50
PWhole Foods Market
LGroceries
^
```

**Errors:**
| Status | Code | When |
|--------|------|------|
| `400` | `INVALID_FORMAT` | Unsupported export format |
| `400` | `INVALID_DATE_RANGE` | `date_from` is after `date_to` |

---

## 11. Settings & i18n

### 11.1 Get User Settings

```
GET /api/settings
```

**Auth:** Required  
**Phase:** P1  

**Response `200`:**
```json
{
  "data": {
    "default_currency": "USD",
    "locale": "en-US",
    "timezone": "America/New_York",
    "date_format": "MM/DD/YYYY",
    "theme": "dark",
    "ai_enabled": false,
    "ai_provider": null,
    "ai_model_text": null,
    "ai_model_vision": null,
    "ai_confidence_threshold": 0.7,
    "ai_receipt_scanning": true,
    "ai_categorization_suggestions": true,
    "ai_import_enrichment": false,
    "ai_payee_normalization": true
  }
}
```

---

### 11.2 Update User Settings

```
PUT /api/settings
```

**Auth:** Required  
**Phase:** P1  

**Request Body:** Partial update — only include fields to change.
```json
{
  "default_currency": "EUR",
  "locale": "de-DE",
  "theme": "light",
  "ai_enabled": true,
  "ai_provider": "ollama"
}
```

**Response `200`:** Updated settings object.

---

### 11.3 List Available Locales

```
GET /api/i18n/locales
```

**Auth:** None  
**Phase:** P1  

**Response `200`:**
```json
{
  "data": [
    {
      "code": "en-US",
      "name": "English (US)",
      "native_name": "English (US)",
      "completeness": 100,
      "is_default": true
    },
    {
      "code": "pl-PL",
      "name": "Polish",
      "native_name": "Polski",
      "completeness": 85,
      "is_default": false
    }
  ]
}
```

---

## 12. AI Features

> All AI endpoints require `ai_enabled = true` in user settings. When disabled, these return `403` with code `AI_DISABLED`.

### 12.1 Scan Receipt

```
POST /api/ai/receipt/scan
```

**Auth:** Required  
**Phase:** AI  
**Content-Type:** `multipart/form-data`

**Request:**
- `image`: binary file (JPEG, PNG, WebP, PDF — max 10 MB)

**Response `200`:**
```json
{
  "data": {
    "merchant": "Whole Foods Market",
    "date": "2026-01-15",
    "total": "42.50",
    "currency": "USD",
    "items": [
      { "description": "Organic Bananas", "amount": "3.99" },
      { "description": "Almond Milk", "amount": "4.49" }
    ],
    "suggested_category_id": "uuid",
    "suggested_category_name": "Groceries",
    "confidence": 0.92,
    "raw_text": "WHOLE FOODS MARKET #10456\n123 Main St..."
  }
}
```

**Errors:**
| Status | Code | When |
|--------|------|------|
| `403` | `AI_DISABLED` | AI features not enabled |
| `413` | `FILE_TOO_LARGE` | Image exceeds 10 MB |
| `503` | `PROVIDER_UNAVAILABLE` | AI provider not reachable |

---

### 12.2 Get Categorization Suggestions

```
GET /api/ai/suggestions/{transaction_id}
```

**Auth:** Required  
**Phase:** AI  

Get AI-suggested category for a specific transaction.

**Response `200`:**
```json
{
  "data": {
    "transaction_id": "uuid",
    "suggested_category_id": "uuid",
    "suggested_category_name": "Subscriptions",
    "confidence": 0.88,
    "reasoning": "Transaction description contains 'SPOTIFY' which is a music streaming subscription service"
  }
}
```

---

### 12.3 Batch Categorize

```
POST /api/ai/categorize/batch
```

**Auth:** Required  
**Phase:** AI  

Batch categorize multiple uncategorized transactions.

**Request Body:**
```json
{
  "transaction_ids": ["uuid1", "uuid2", "uuid3"],
  "auto_apply_above_threshold": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `transaction_ids` | uuid[] | — | 1–100 transaction IDs |
| `auto_apply_above_threshold` | bool | `false` | Automatically apply suggestions with confidence >= threshold |

**Response `200`:**
```json
{
  "data": {
    "suggestions": [
      {
        "transaction_id": "uuid1",
        "suggested_category_id": "uuid",
        "suggested_category_name": "Subscriptions",
        "confidence": 0.92,
        "auto_applied": false
      },
      {
        "transaction_id": "uuid2",
        "suggested_category_id": null,
        "confidence": 0.0,
        "auto_applied": false
      }
    ],
    "auto_applied_count": 0,
    "processing_time_ms": 1250
  }
}
```

---

### 12.4 Normalize Payees

```
POST /api/ai/normalize/payees
```

**Auth:** Required  
**Phase:** AI  

Normalize raw bank payee strings into clean merchant names.

**Request Body:**
```json
{
  "payees": [
    "SPOTIFY AB STOCKHOLM SE",
    "AMZN MKTP US*1A2B3C4D5",
    "SQ *COFFEE SHOP NYC"
  ],
  "auto_apply": false
}
```

**Response `200`:**
```json
{
  "data": {
    "normalizations": [
      { "original": "SPOTIFY AB STOCKHOLM SE", "normalized": "Spotify" },
      { "original": "AMZN MKTP US*1A2B3C4D5", "normalized": "Amazon" },
      { "original": "SQ *COFFEE SHOP NYC", "normalized": "Coffee Shop (Square)" }
    ]
  }
}
```

---

### 12.5 AI Status

```
GET /api/ai/status
```

**Auth:** Required  
**Phase:** AI  

Check AI module health.

**Response `200`:**
```json
{
  "data": {
    "enabled": true,
    "provider": "ollama",
    "provider_status": "available",
    "text_model": "phi3.5:mini",
    "vision_model": "smolvlm:2b",
    "text_model_available": true,
    "vision_model_available": true,
    "features": {
      "receipt_scanning": true,
      "categorization": true,
      "payee_normalization": true,
      "import_enrichment": false
    }
  }
}
```

---

### 12.6 List Models

```
GET /api/ai/models
```

**Auth:** Required  
**Phase:** AI  

List available models from the configured provider.

**Response `200`:**
```json
{
  "data": [
    {
      "id": "phi3.5:mini",
      "name": "Phi 3.5 Mini",
      "size_bytes": 2200000000,
      "supports_vision": false,
      "is_text_default": true,
      "is_vision_default": false
    },
    {
      "id": "smolvlm:2b",
      "name": "SmolVLM 2B",
      "size_bytes": 1800000000,
      "supports_vision": true,
      "is_text_default": false,
      "is_vision_default": true
    }
  ]
}
```

---

## 13. Admin

> All admin endpoints require `role = admin`.

### 13.1 List Users

```
GET /api/admin/users
```

**Auth:** Required (admin only)  
**Phase:** P7  

**Response `200`:**
```json
{
  "data": [
    {
      "id": "uuid",
      "username": "john",
      "email": "john@example.com",
      "role": "admin",
      "locale": "en-US",
      "created_at": "2026-01-01T12:00:00Z",
      "last_login_at": "2026-03-02T08:00:00Z",
      "accounts_count": 3,
      "transactions_count": 1250
    }
  ]
}
```

---

### 13.2 Create Backup

```
POST /api/admin/backup
```

**Auth:** Required (admin only)  
**Phase:** P7  

**Request Body:**
```json
{
  "encryption_key": "user-provided-passphrase"
}
```

**Response `200`:**
Returns the encrypted database dump as a binary download.

**Headers:**
```
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="rustvault-backup-2026-03-02.enc"
```

---

### 13.3 Restore Backup

```
POST /api/admin/restore
```

**Auth:** Required (admin only)  
**Phase:** P7  
**Content-Type:** `multipart/form-data`

**Request:**
- `file`: encrypted backup file
- `encryption_key`: passphrase to decrypt

**Response `200`:**
```json
{
  "data": {
    "status": "restored",
    "users_restored": 2,
    "accounts_restored": 5,
    "transactions_restored": 3400
  }
}
```

---

## 14. System

### 14.1 Health Check

```
GET /api/health
```

**Auth:** None  
**Phase:** P0  

For container orchestration. Does not leak version or internal info.

**Response `200`:**
```json
{
  "status": "healthy"
}
```

Returns `503` if the database is unreachable.

---

### 14.2 API Documentation

```
GET /api/docs
```

**Auth:** None  
**Phase:** P7  

Serves the Scalar interactive API documentation UI. Can be disabled via `config.toml` (`docs.serve_api_docs = false`).

---

### 14.3 OpenAPI Spec

```
GET /api/docs/openapi.json
```

**Auth:** None  
**Phase:** P7  

Returns the raw OpenAPI 3.1 specification in JSON format.

---

## 15. WebSocket

### 15.1 Real-Time Events

```
WS /api/ws
```

**Auth:** JWT in initial handshake (query param `?token=<access_token>`)  
**Phase:** P7  

Pushes real-time events:

**Event Types:**
```json
// Import progress
{
  "type": "import_progress",
  "data": {
    "import_id": "uuid",
    "progress": 0.65,
    "rows_processed": 101,
    "total_rows": 156
  }
}

// Dashboard invalidation
{
  "type": "dashboard_invalidated",
  "data": {
    "reason": "transaction_created"
  }
}

// Transaction created/updated/deleted
{
  "type": "transaction_updated",
  "data": {
    "transaction_id": "uuid",
    "action": "update"
  }
}

// Budget alert
{
  "type": "budget_alert",
  "data": {
    "budget_id": "uuid",
    "category_name": "Entertainment",
    "percent_used": 95.0
  }
}
```

---

## 16. Implementation Phases

### Phase 0 — Skeleton
| Endpoint | Method | Path |
|----------|--------|------|
| Health Check | `GET` | `/api/health` |

### Phase 1 — Core Backend
| Endpoint | Method | Path |
|----------|--------|------|
| Register | `POST` | `/api/auth/register` |
| Login | `POST` | `/api/auth/login` |
| Refresh | `POST` | `/api/auth/refresh` |
| Current User | `GET` | `/api/auth/me` |
| Logout | `POST` | `/api/auth/logout` |
| Change Password | `POST` | `/api/auth/change-password` |
| List Banks | `GET` | `/api/banks` |
| Create Bank | `POST` | `/api/banks` |
| Update Bank | `PUT` | `/api/banks/{id}` |
| Archive Bank | `PUT` | `/api/banks/{id}/archive` |
| List Accounts | `GET` | `/api/accounts` |
| Create Account | `POST` | `/api/accounts` |
| Update Account | `PUT` | `/api/accounts/{id}` |
| Archive Account | `PUT` | `/api/accounts/{id}/archive` |
| List Categories | `GET` | `/api/categories` |
| Create Category | `POST` | `/api/categories` |
| Update Category | `PUT` | `/api/categories/{id}` |
| Delete Category | `DELETE` | `/api/categories/{id}` |
| Bulk Create Categories | `POST` | `/api/categories/bulk` |
| List Tags | `GET` | `/api/tags` |
| Create Tag | `POST` | `/api/tags` |
| Update Tag | `PUT` | `/api/tags/{id}` |
| Delete Tag | `DELETE` | `/api/tags/{id}` |
| Bulk Create Tags | `POST` | `/api/tags/bulk` |
| Get Settings | `GET` | `/api/settings` |
| Update Settings | `PUT` | `/api/settings` |
| List Locales | `GET` | `/api/i18n/locales` |

**Total: 26 endpoints**

### Phase 2 — Web UI Shell (backend additions)
| Endpoint | Method | Path |
|----------|--------|------|
| List Sessions | `GET` | `/api/auth/sessions` |
| Revoke Session | `DELETE` | `/api/auth/sessions/{id}` |

**Total: 2 endpoints**

### Phase 3 — Transactions, Transfers & Import
| Endpoint | Method | Path |
|----------|--------|------|
| List Transactions | `GET` | `/api/transactions` |
| Get Transaction | `GET` | `/api/transactions/{id}` |
| Create Transaction | `POST` | `/api/transactions` |
| Update Transaction | `PUT` | `/api/transactions/{id}` |
| Delete Transaction | `DELETE` | `/api/transactions/{id}` |
| Bulk Update | `PATCH` | `/api/transactions/bulk` |
| Create Transfer | `POST` | `/api/transfers` |
| Link Transactions | `POST` | `/api/transfers/link` |
| Unlink Transfer | `DELETE` | `/api/transfers/{transfer_id}` |
| Detect Transfers | `POST` | `/api/transfers/detect` |
| Upload File | `POST` | `/api/import/upload` |
| Configure Import | `POST` | `/api/import/configure` |
| Execute Import | `POST` | `/api/import/execute` |
| List Imports | `GET` | `/api/imports` |
| Get Import | `GET` | `/api/imports/{id}` |
| Rollback Import | `DELETE` | `/api/imports/{id}` |
| List Rules | `GET` | `/api/rules` |
| Create Rule | `POST` | `/api/rules` |
| Update Rule | `PUT` | `/api/rules/{id}` |
| Delete Rule | `DELETE` | `/api/rules/{id}` |
| Test Rule | `POST` | `/api/rules/test` |
| Re-run Rules | `POST` | `/api/rules/rerun` |

**Total: 22 endpoints**

### Phase 4 — Budgets
| Endpoint | Method | Path |
|----------|--------|------|
| List Budgets | `GET` | `/api/budgets` |
| Create Budget | `POST` | `/api/budgets` |
| Get Budget | `GET` | `/api/budgets/{id}` |
| Update Budget | `PUT` | `/api/budgets/{id}` |
| Delete Budget | `DELETE` | `/api/budgets/{id}` |
| Add Budget Line | `POST` | `/api/budgets/{id}/lines` |
| Update Budget Line | `PUT` | `/api/budgets/{id}/lines/{line_id}` |
| Delete Budget Line | `DELETE` | `/api/budgets/{id}/lines/{line_id}` |
| Bulk Set Lines | `POST` | `/api/budgets/{id}/lines/bulk` |
| Budget Summary | `GET` | `/api/budgets/{id}/summary` |
| Copy Budget | `POST` | `/api/budgets/{id}/copy` |

**Total: 11 endpoints**

### Phase 5 — Reports
| Endpoint | Method | Path |
|----------|--------|------|
| Dashboard Summary | `GET` | `/api/reports/summary` |
| Income vs Expense | `GET` | `/api/reports/income-expense` |
| Category Trend | `GET` | `/api/reports/category/{id}/trend` |
| Balance History | `GET` | `/api/reports/balance-history` |
| Cash Flow | `GET` | `/api/reports/cash-flow` |
| Export Transactions | `GET` | `/api/export/transactions` |

**Total: 6 endpoints**

### Phase 7 — Polish & Admin
| Endpoint | Method | Path |
|----------|--------|------|
| List Users (admin) | `GET` | `/api/admin/users` |
| Create Backup (admin) | `POST` | `/api/admin/backup` |
| Restore Backup (admin) | `POST` | `/api/admin/restore` |
| API Docs UI | `GET` | `/api/docs` |
| OpenAPI Spec | `GET` | `/api/docs/openapi.json` |
| WebSocket | `WS` | `/api/ws` |

**Total: 6 endpoints**

### AI Features (parallel, after P3)
| Endpoint | Method | Path |
|----------|--------|------|
| Scan Receipt | `POST` | `/api/ai/receipt/scan` |
| Get Suggestions | `GET` | `/api/ai/suggestions/{transaction_id}` |
| Batch Categorize | `POST` | `/api/ai/categorize/batch` |
| Normalize Payees | `POST` | `/api/ai/normalize/payees` |
| AI Status | `GET` | `/api/ai/status` |
| List Models | `GET` | `/api/ai/models` |

**Total: 6 endpoints**

---

## 17. Endpoint Summary Table

| # | Method | Path | Auth | Rate Limit | Phase |
|---|--------|------|------|-----------|-------|
| 1 | `GET` | `/api/health` | No | Global | P0 |
| 2 | `POST` | `/api/auth/register` | No | 5/15min/IP | P1 |
| 3 | `POST` | `/api/auth/login` | No | 5/15min/IP | P1 |
| 4 | `POST` | `/api/auth/refresh` | Cookie | 5/15min/IP | P1 |
| 5 | `GET` | `/api/auth/me` | Bearer | Global | P1 |
| 6 | `POST` | `/api/auth/logout` | Bearer | Global | P1 |
| 7 | `POST` | `/api/auth/change-password` | Bearer | 5/15min/IP | P1 |
| 8 | `GET` | `/api/auth/sessions` | Bearer | Global | P2 |
| 9 | `DELETE` | `/api/auth/sessions/{id}` | Bearer | Global | P2 |
| 9a | `GET` | `/api/auth/oidc/config` | No | Global | P1 |
| 9b | `GET` | `/api/auth/oidc/authorize` | No | 5/15min/IP | P1 |
| 9c | `GET` | `/api/auth/oidc/callback` | No | 5/15min/IP | P1 |
| 10 | `GET` | `/api/banks` | Bearer | Global | P1 |
| 11 | `POST` | `/api/banks` | Bearer | Global | P1 |
| 12 | `PUT` | `/api/banks/{id}` | Bearer | Global | P1 |
| 13 | `PUT` | `/api/banks/{id}/archive` | Bearer | Global | P1 |
| 14 | `GET` | `/api/accounts` | Bearer | Global | P1 |
| 15 | `POST` | `/api/accounts` | Bearer | Global | P1 |
| 16 | `PUT` | `/api/accounts/{id}` | Bearer | Global | P1 |
| 17 | `PUT` | `/api/accounts/{id}/archive` | Bearer | Global | P1 |
| 18 | `GET` | `/api/categories` | Bearer | Global | P1 |
| 19 | `POST` | `/api/categories` | Bearer | Global | P1 |
| 20 | `PUT` | `/api/categories/{id}` | Bearer | Global | P1 |
| 21 | `DELETE` | `/api/categories/{id}` | Bearer | Global | P1 |
| 22 | `POST` | `/api/categories/bulk` | Bearer | Global | P1 |
| 23 | `GET` | `/api/tags` | Bearer | Global | P1 |
| 24 | `POST` | `/api/tags` | Bearer | Global | P1 |
| 25 | `PUT` | `/api/tags/{id}` | Bearer | Global | P1 |
| 26 | `DELETE` | `/api/tags/{id}` | Bearer | Global | P1 |
| 27 | `POST` | `/api/tags/bulk` | Bearer | Global | P1 |
| 28 | `GET` | `/api/settings` | Bearer | Global | P1 |
| 29 | `PUT` | `/api/settings` | Bearer | Global | P1 |
| 30 | `GET` | `/api/i18n/locales` | No | Global | P1 |
| 31 | `GET` | `/api/transactions` | Bearer | Global | P3 |
| 32 | `GET` | `/api/transactions/{id}` | Bearer | Global | P3 |
| 33 | `POST` | `/api/transactions` | Bearer | Global | P3 |
| 34 | `PUT` | `/api/transactions/{id}` | Bearer | Global | P3 |
| 35 | `DELETE` | `/api/transactions/{id}` | Bearer | Global | P3 |
| 36 | `PATCH` | `/api/transactions/bulk` | Bearer | Global | P3 |
| 37 | `POST` | `/api/transfers` | Bearer | Global | P3 |
| 38 | `POST` | `/api/transfers/link` | Bearer | Global | P3 |
| 39 | `DELETE` | `/api/transfers/{transfer_id}` | Bearer | Global | P3 |
| 40 | `POST` | `/api/transfers/detect` | Bearer | 10/min/user | P3 |
| 41 | `POST` | `/api/import/upload` | Bearer | 10/hour/user | P3 |
| 42 | `POST` | `/api/import/configure` | Bearer | Global | P3 |
| 43 | `POST` | `/api/import/execute` | Bearer | 10/hour/user | P3 |
| 44 | `GET` | `/api/imports` | Bearer | Global | P3 |
| 45 | `GET` | `/api/imports/{id}` | Bearer | Global | P3 |
| 46 | `DELETE` | `/api/imports/{id}` | Bearer | Global | P3 |
| 47 | `GET` | `/api/rules` | Bearer | Global | P3 |
| 48 | `POST` | `/api/rules` | Bearer | Global | P3 |
| 49 | `PUT` | `/api/rules/{id}` | Bearer | Global | P3 |
| 50 | `DELETE` | `/api/rules/{id}` | Bearer | Global | P3 |
| 51 | `POST` | `/api/rules/test` | Bearer | Global | P3 |
| 52 | `POST` | `/api/rules/rerun` | Bearer | Global | P3 |
| 53 | `GET` | `/api/budgets` | Bearer | Global | P4 |
| 54 | `POST` | `/api/budgets` | Bearer | Global | P4 |
| 55 | `GET` | `/api/budgets/{id}` | Bearer | Global | P4 |
| 56 | `PUT` | `/api/budgets/{id}` | Bearer | Global | P4 |
| 57 | `DELETE` | `/api/budgets/{id}` | Bearer | Global | P4 |
| 58 | `POST` | `/api/budgets/{id}/lines` | Bearer | Global | P4 |
| 59 | `PUT` | `/api/budgets/{id}/lines/{line_id}` | Bearer | Global | P4 |
| 60 | `DELETE` | `/api/budgets/{id}/lines/{line_id}` | Bearer | Global | P4 |
| 61 | `POST` | `/api/budgets/{id}/lines/bulk` | Bearer | Global | P4 |
| 62 | `GET` | `/api/budgets/{id}/summary` | Bearer | 20/min/user | P4 |
| 63 | `POST` | `/api/budgets/{id}/copy` | Bearer | Global | P4 |
| 64 | `GET` | `/api/reports/summary` | Bearer | 20/min/user | P5 |
| 65 | `GET` | `/api/reports/income-expense` | Bearer | 20/min/user | P5 |
| 66 | `GET` | `/api/reports/category/{id}/trend` | Bearer | 20/min/user | P5 |
| 67 | `GET` | `/api/reports/balance-history` | Bearer | 20/min/user | P5 |
| 68 | `GET` | `/api/reports/cash-flow` | Bearer | 20/min/user | P5 |
| 69 | `GET` | `/api/export/transactions` | Bearer | 10/hour/user | P5 |
| 70 | `POST` | `/api/ai/receipt/scan` | Bearer | Global | AI |
| 71 | `GET` | `/api/ai/suggestions/{transaction_id}` | Bearer | Global | AI |
| 72 | `POST` | `/api/ai/categorize/batch` | Bearer | Global | AI |
| 73 | `POST` | `/api/ai/normalize/payees` | Bearer | Global | AI |
| 74 | `GET` | `/api/ai/status` | Bearer | Global | AI |
| 75 | `GET` | `/api/ai/models` | Bearer | Global | AI |
| 76 | `GET` | `/api/admin/users` | Admin | Global | P7 |
| 77 | `POST` | `/api/admin/backup` | Admin | Global | P7 |
| 78 | `POST` | `/api/admin/restore` | Admin | Global | P7 |
| 79 | `GET` | `/api/docs` | No | Global | P7 |
| 80 | `GET` | `/api/docs/openapi.json` | No | Global | P7 |
| 81 | `WS` | `/api/ws` | Bearer (query) | — | P7 |

**Total: 81 endpoints across 7 phases**
