# Authentication Architecture

> Detailed authentication and session management flows for RustVault.
> Covers local auth (password + JWT), OIDC/SSO, token lifecycle, and session management.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Auth Strategy Summary](#2-auth-strategy-summary)
3. [Registration Flow](#3-registration-flow)
4. [Login Flow (Local)](#4-login-flow-local)
5. [OIDC / SSO Login Flow](#5-oidc--sso-login-flow)
6. [Token Lifecycle](#6-token-lifecycle)
7. [Token Refresh Flow](#7-token-refresh-flow)
8. [Logout & Session Revocation](#8-logout--session-revocation)
9. [Session Management](#9-session-management)
10. [Password Change Flow](#10-password-change-flow)
11. [Security Controls](#11-security-controls)
12. [Mobile Authentication](#12-mobile-authentication)
13. [Token Storage Model](#13-token-storage-model)

---

## 1. Overview

RustVault uses a **dual-token architecture** for authentication:

- **Access token** (JWT) — short-lived (15 min), stored in browser memory, sent in `Authorization` header
- **Refresh token** — longer-lived (7 days), stored in an `HttpOnly; Secure; SameSite=Strict` cookie, single-use with rotation

This design balances security (short exposure window) with UX (no frequent re-login).

```mermaid
graph LR
    subgraph Tokens["Token Architecture"]
        AT["Access Token (JWT)<br/>─────────────────<br/>TTL: 15 min<br/>Storage: Memory only<br/>Transport: Authorization header<br/>Revocable: No (short-lived)"]
        RT["Refresh Token<br/>─────────────────<br/>TTL: 7 days<br/>Storage: HttpOnly cookie<br/>Transport: Cookie (auto)<br/>Revocable: Yes (DB allowlist)"]
    end

    AT -->|"expires"| REFRESH["Refresh Endpoint<br/>POST /api/auth/refresh"]
    REFRESH -->|"rotates"| RT
    REFRESH -->|"issues new"| AT

    style AT fill:#e8f4fd,stroke:#4a90d9
    style RT fill:#fff8e1,stroke:#f0ad4e
```

---

## 2. Auth Strategy Summary

| Aspect | Design Decision | Rationale |
|--------|----------------|-----------|
| **Access token format** | JWT (HS256) | Stateless verification at middleware layer — no DB call per request |
| **Access token storage** | JavaScript variable (SPA memory) | Not accessible to XSS via `document.cookie` or `localStorage` |
| **Access token TTL** | 15 minutes | Limits exposure window if token is leaked |
| **Refresh token format** | Opaque random string | No sensitive claims to decode; server-side validation only |
| **Refresh token storage** | `HttpOnly; Secure; SameSite=Strict` cookie | Inaccessible to JavaScript, not sent cross-origin |
| **Refresh token TTL** | 7 days | Balances security with "stay logged in" UX |
| **Refresh token rotation** | Single-use (each refresh issues a new token) | Detects token theft — reuse triggers full session revocation |
| **Password hashing** | Argon2id (19 MiB, 2 iterations, 1 parallelism) | Memory-hard — resistant to GPU/ASIC attacks |
| **OIDC support** | Authorization Code Flow + PKCE | Industry standard for SPAs, prevents authorization code interception |
| **CSRF protection** | `SameSite=Strict` + `X-Requested-With` header | Defense-in-depth — cookie not sent cross-origin, custom header blocks simple CORS |

> For the reasoning behind choosing JWTs over session cookies, see [ADR-0008: Auth & JWT Design](../adr/0008-auth-jwt-design.md).

---

## 3. Registration Flow

```mermaid
sequenceDiagram
    actor User
    participant SPA as SolidJS SPA
    participant API as POST /api/auth/register
    participant Auth as Auth Service
    participant DB as PostgreSQL

    User->>SPA: Fill registration form
    SPA->>SPA: Client-side validation<br/>(length, format)

    SPA->>API: POST /api/auth/register<br/>{username, email, password}<br/>+ X-Requested-With: RustVault

    API->>API: Rate limit check (5/15min/IP)
    
    alt Rate limited
        API-->>SPA: 429 rate_limited
    end

    API->>Auth: Validate input
    Auth->>Auth: Check password policy<br/>(min 10 chars, max 128)
    Auth->>Auth: HaveIBeenPwned check<br/>(k-anonymity API)
    
    alt Password breached
        Auth-->>API: Password found in breach database
        API-->>SPA: 400 password_breached
    end

    Auth->>DB: SELECT WHERE email = $1 OR username = $2
    
    alt Already exists
        Auth-->>API: Conflict
        API-->>SPA: 409 conflict
    end

    Auth->>Auth: Hash password (Argon2id)
    Auth->>DB: INSERT INTO users (...)
    Auth->>Auth: Generate access_token (JWT, 15 min)
    Auth->>Auth: Generate refresh_token (random, 7 days)
    Auth->>DB: INSERT INTO refresh_tokens (hash, user_id, ...)
    Auth-->>API: User + tokens

    API-->>SPA: 201 {user, access_token, expires_in}<br/>Set-Cookie: refresh_token=<opaque>;<br/>HttpOnly; Secure; SameSite=Strict; Path=/api

    SPA->>SPA: Store access_token in memory<br/>Schedule refresh at expires_in - 60s
    SPA->>SPA: Redirect to dashboard

    Note over SPA: First user gets role: admin<br/>Subsequent users get role: member
```

---

## 4. Login Flow (Local)

```mermaid
sequenceDiagram
    actor User
    participant SPA as SolidJS SPA
    participant API as POST /api/auth/login
    participant Auth as Auth Service
    participant DB as PostgreSQL

    User->>SPA: Enter email + password
    SPA->>API: POST /api/auth/login<br/>{email, password}<br/>+ X-Requested-With: RustVault

    API->>API: Rate limit check (5/15min/IP)

    alt Rate limited
        API-->>SPA: 429 rate_limited<br/>{retry_after: seconds}
    end

    API->>Auth: Authenticate
    Auth->>DB: SELECT user WHERE email = $1
    
    alt User not found
        Auth->>Auth: Dummy Argon2id hash<br/>(constant-time, prevents timing attack)
        Auth-->>API: Invalid credentials
        API-->>SPA: 401 invalid_credentials
    end

    Auth->>Auth: Argon2id verify(password, stored_hash)

    alt Wrong password
        Auth->>DB: INCREMENT failed_login_count
        alt Lockout threshold (20) reached
            Auth->>DB: SET locked_until = NOW() + 1 hour
            Auth-->>API: Account locked
            API-->>SPA: 403 account_locked
        end
        Auth-->>API: Invalid credentials
        API-->>SPA: 401 invalid_credentials
    end

    Auth->>DB: RESET failed_login_count
    Auth->>Auth: Generate access_token (JWT)
    Auth->>Auth: Generate refresh_token (random)
    Auth->>DB: INSERT INTO refresh_tokens<br/>(token_hash, user_id, ip, user_agent, expires_at)
    Auth-->>API: User + tokens

    API-->>SPA: 200 {user, access_token, expires_in}<br/>Set-Cookie: refresh_token=<opaque>;<br/>HttpOnly; Secure; SameSite=Strict; Path=/api

    SPA->>SPA: Store access_token in memory
    SPA->>SPA: Start refresh timer

    Note over Auth,DB: Login event logged to audit_log<br/>(user_id, ip, user_agent, timestamp)
```

---

## 5. OIDC / SSO Login Flow

```mermaid
sequenceDiagram
    actor User
    participant SPA as SolidJS SPA
    participant API as RustVault API
    participant OIDC as OIDC Provider<br/>(Authentik / Keycloak)
    participant DB as PostgreSQL

    User->>SPA: Click "Login with SSO"
    SPA->>API: GET /api/auth/oidc/authorize

    API->>API: Generate state (CSRF token)
    API->>API: Generate code_verifier + code_challenge (PKCE)
    API->>API: Store state + code_verifier<br/>(encrypted HttpOnly cookie or server-side)

    API-->>SPA: 302 Redirect to OIDC Provider<br/>?client_id=rustvault<br/>&redirect_uri=.../callback<br/>&response_type=code<br/>&scope=openid profile email<br/>&state=<csrf><br/>&code_challenge=<S256><br/>&code_challenge_method=S256

    SPA->>OIDC: Follow redirect
    User->>OIDC: Authenticate (username/password, MFA, etc.)
    OIDC-->>SPA: 302 Redirect to callback<br/>?code=<auth_code>&state=<csrf>

    SPA->>API: GET /api/auth/oidc/callback<br/>?code=<auth_code>&state=<csrf>

    API->>API: Verify state matches stored CSRF token
    
    alt State mismatch
        API-->>SPA: 400 invalid_state (possible CSRF)
    end

    API->>OIDC: POST /token<br/>{grant_type: authorization_code,<br/>code, redirect_uri, client_id,<br/>client_secret, code_verifier}
    OIDC-->>API: {access_token, id_token, refresh_token}

    API->>API: Validate id_token<br/>(signature via JWKS, iss, aud, exp, nonce)
    API->>API: Extract claims (sub, email, name)

    API->>DB: SELECT user WHERE oidc_subject = $1<br/>OR email = $2

    alt New user (auto_register = true)
        API->>DB: INSERT user<br/>(oidc_subject, email, name, auth_provider: oidc)
    else Existing user (link)
        API->>DB: UPDATE user SET oidc_subject = $1,<br/>auth_provider = 'both'
    else auto_register = false & not found
        API-->>SPA: 403 oidc_registration_disabled
    end

    API->>API: Generate RustVault access_token (JWT)
    API->>API: Generate RustVault refresh_token
    API->>DB: INSERT refresh_token

    API-->>SPA: 302 Redirect to /<br/>Set-Cookie: refresh_token=<opaque>

    SPA->>SPA: Extract access_token from response
    SPA->>SPA: Navigate to dashboard

    Note over API,OIDC: OIDC provider tokens are NOT stored.<br/>RustVault issues its own JWT after OIDC validation.<br/>Subsequent API calls use RustVault's JWT only.
```

---

## 6. Token Lifecycle

### Access Token (JWT)

```mermaid
stateDiagram-v2
    [*] --> Issued: Login / Register / Refresh
    Issued --> Valid: JWT created (15 min TTL)
    Valid --> Expired: Clock > exp claim
    Valid --> Used: Attached to API request
    Used --> Verified: Signature + exp + claims OK
    Verified --> Authorized: user_id extracted
    Expired --> RefreshNeeded: SPA detects expiry
    RefreshNeeded --> Issued: POST /api/auth/refresh
    RefreshNeeded --> LoggedOut: Refresh fails
    LoggedOut --> [*]
```

#### JWT Claims (Access Token Payload)

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "role": "admin",
  "iat": 1709463600,
  "exp": 1709464500
}
```

| Claim | Type | Description |
|-------|------|-------------|
| `sub` | UUID | User ID |
| `role` | string | `admin` or `member` |
| `iat` | timestamp | Issued at (Unix seconds) |
| `exp` | timestamp | Expires at (iat + 900s = 15 min) |

**Not included in JWT:** email, username, settings — fetched via `GET /api/auth/me` and cached in SPA.

### Refresh Token

```mermaid
stateDiagram-v2
    [*] --> Generated: Random 256-bit value
    Generated --> Stored: SHA-256 hash saved in DB<br/>Original sent as HttpOnly cookie
    Stored --> Presented: Browser sends cookie to /api/auth/refresh
    Presented --> Validated: Hash matches DB record + not expired
    Validated --> Consumed: Token marked as used
    Consumed --> NewIssued: New refresh_token + access_token issued
    NewIssued --> Stored: New token replaces old

    Presented --> TheftDetected: Token already consumed (reuse!)
    TheftDetected --> AllRevoked: ALL user sessions revoked
    AllRevoked --> [*]

    Stored --> Expired: TTL exceeded (7 days)
    Expired --> [*]
    Stored --> Revoked: User logout / admin action
    Revoked --> [*]
```

#### Refresh Token DB Schema

```sql
CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  BYTEA NOT NULL,          -- SHA-256 of the token
    ip_address  INET,
    user_agent  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,             -- NULL = active, set on use
    revoked_at  TIMESTAMPTZ              -- NULL = not revoked
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
```

---

## 7. Token Refresh Flow

```mermaid
sequenceDiagram
    participant SPA as SolidJS SPA
    participant API as POST /api/auth/refresh
    participant Auth as Auth Service
    participant DB as PostgreSQL

    Note over SPA: Access token expired<br/>(or 60s before expiry)

    SPA->>API: POST /api/auth/refresh<br/>Cookie: refresh_token=<token_value>

    API->>Auth: Extract token from cookie
    Auth->>Auth: SHA-256 hash the token
    Auth->>DB: SELECT FROM refresh_tokens<br/>WHERE token_hash = $1<br/>AND expires_at > NOW()<br/>AND revoked_at IS NULL

    alt Token not found or expired
        Auth-->>API: Invalid token
        API-->>SPA: 401 token_expired
        SPA->>SPA: Redirect to /login
    end

    alt Token already consumed (consumed_at IS NOT NULL)
        Note over Auth: TOKEN REUSE DETECTED!<br/>Possible theft scenario
        Auth->>DB: UPDATE refresh_tokens<br/>SET revoked_at = NOW()<br/>WHERE user_id = $1
        Note over Auth: ALL sessions for this user revoked
        Auth-->>API: Token reuse detected
        API-->>SPA: 401 token_invalid<br/>{code: "token_reuse_detected"}
        SPA->>SPA: Redirect to /login<br/>Show security warning
    end

    Auth->>DB: UPDATE refresh_tokens<br/>SET consumed_at = NOW()<br/>WHERE id = $1
    
    Auth->>Auth: Generate new access_token (JWT, 15 min)
    Auth->>Auth: Generate new refresh_token (random)
    Auth->>DB: INSERT INTO refresh_tokens<br/>(new token_hash, user_id, ...)

    Auth-->>API: New access_token + refresh_token
    API-->>SPA: 200 {access_token, expires_in}<br/>Set-Cookie: refresh_token=<new_token>;<br/>HttpOnly; Secure; SameSite=Strict; Path=/api

    SPA->>SPA: Store new access_token in memory
    SPA->>SPA: Reset refresh timer
```

### Refresh Token Reuse Detection

Token reuse indicates a potential theft scenario:

```mermaid
sequenceDiagram
    actor Attacker
    actor User
    participant API as RustVault API
    participant DB as PostgreSQL

    Note over User: User logs in normally
    API->>DB: Store refresh_token_A (hash)

    Note over Attacker: Attacker steals refresh_token_A<br/>(e.g., via XSS on a different domain,<br/>network interception, malware)

    User->>API: POST /refresh (token_A)
    API->>DB: Mark token_A consumed
    API->>DB: Store token_B (new)
    API-->>User: New access + refresh tokens (B)

    Note over Attacker: Later, attacker tries to use stolen token_A
    Attacker->>API: POST /refresh (token_A)
    API->>DB: token_A found but consumed_at IS NOT NULL

    Note over API: REUSE DETECTED!
    API->>DB: REVOKE ALL tokens for this user_id
    API-->>Attacker: 401 token_invalid

    Note over User: User's session also invalidated<br/>Must re-authenticate (safe side)
```

---

## 8. Logout & Session Revocation

### Single Session Logout

```mermaid
sequenceDiagram
    actor User
    participant SPA as SolidJS SPA
    participant API as POST /api/auth/logout
    participant Auth as Auth Service
    participant DB as PostgreSQL

    User->>SPA: Click "Log out"
    SPA->>API: POST /api/auth/logout<br/>Authorization: Bearer <access_token><br/>Cookie: refresh_token=<token>

    API->>Auth: Extract refresh_token from cookie
    Auth->>Auth: SHA-256 hash the token
    Auth->>DB: UPDATE refresh_tokens<br/>SET revoked_at = NOW()<br/>WHERE token_hash = $1

    Auth-->>API: Success
    API-->>SPA: 204 No Content<br/>Clear-Cookie: refresh_token=; Max-Age=0

    SPA->>SPA: Clear access_token from memory
    SPA->>SPA: Clear cached user data
    SPA->>SPA: Redirect to /login
```

### Revoke All Sessions ("Log Out Everywhere")

```mermaid
sequenceDiagram
    actor User
    participant SPA as SolidJS SPA
    participant API as DELETE /api/auth/sessions
    participant Auth as Auth Service
    participant DB as PostgreSQL

    User->>SPA: Click "Log out everywhere"
    SPA->>API: DELETE /api/auth/sessions<br/>Authorization: Bearer <access_token>

    API->>Auth: Get user_id from JWT
    Auth->>DB: UPDATE refresh_tokens<br/>SET revoked_at = NOW()<br/>WHERE user_id = $1<br/>AND revoked_at IS NULL

    Note over DB: All sessions revoked.<br/>Other devices will fail<br/>on next refresh attempt.

    Auth-->>API: Revoked count
    API-->>SPA: 200 {revoked_sessions: 5}<br/>Clear-Cookie: refresh_token=; Max-Age=0

    SPA->>SPA: Clear tokens + redirect to /login

    Note over User: Access tokens on other devices<br/>remain valid until expiry (max 15 min).<br/>Refresh will fail → forced re-login.
```

---

## 9. Session Management

### Session List UI Flow

```mermaid
sequenceDiagram
    actor User
    participant SPA as SolidJS SPA
    participant API as RustVault API
    participant DB as PostgreSQL

    User->>SPA: Navigate to Settings → Sessions
    SPA->>API: GET /api/auth/sessions<br/>Authorization: Bearer <token>

    API->>DB: SELECT id, ip_address, user_agent,<br/>created_at, last_used_at<br/>FROM refresh_tokens<br/>WHERE user_id = $1<br/>AND revoked_at IS NULL<br/>AND expires_at > NOW()

    DB-->>API: Active sessions
    API-->>SPA: 200 {data: [{id, ip, user_agent, created_at, last_used_at, is_current}, ...]}

    SPA->>SPA: Render session list<br/>Highlight current session<br/>Show "Revoke" button on others

    User->>SPA: Click "Revoke" on suspicious session
    SPA->>API: DELETE /api/auth/sessions/{id}

    API->>DB: UPDATE refresh_tokens<br/>SET revoked_at = NOW()<br/>WHERE id = $1 AND user_id = $2

    API-->>SPA: 204 No Content
    SPA->>SPA: Remove session from list
```

### Session Data Model

```
┌──────────────────────────────────────────────────────────┐
│ Active Sessions                                          │
├──────────────────────────────────────────────────────────┤
│ 🟢 Current Session                                      │
│   Chrome on macOS · 192.168.1.10                         │
│   Last active: Just now                                  │
│                                                          │
│ 🔵 Firefox on Windows · 10.0.0.5          [Revoke]      │
│   Last active: 2 hours ago                               │
│                                                          │
│ 🔵 RustVault iOS · 192.168.1.20           [Revoke]      │
│   Last active: 1 day ago                                 │
│                                                          │
│                            [Log out everywhere]          │
└──────────────────────────────────────────────────────────┘
```

---

## 10. Password Change Flow

```mermaid
sequenceDiagram
    actor User
    participant SPA as SolidJS SPA
    participant API as POST /api/auth/change-password
    participant Auth as Auth Service
    participant DB as PostgreSQL

    User->>SPA: Enter current + new password
    SPA->>SPA: Validate new password (min 10 chars)

    SPA->>API: POST /api/auth/change-password<br/>{current_password, new_password}<br/>Authorization: Bearer <token>

    API->>Auth: Verify current password
    Auth->>DB: SELECT password_hash FROM users<br/>WHERE id = $1
    Auth->>Auth: Argon2id verify(current_password, hash)

    alt Current password wrong
        Auth-->>API: Wrong password
        API-->>SPA: 401 invalid_credentials
    end

    Auth->>Auth: Validate new password policy
    Auth->>Auth: HaveIBeenPwned check
    Auth->>Auth: Hash new password (Argon2id)

    Auth->>DB: UPDATE users<br/>SET password_hash = $1<br/>WHERE id = $2

    Auth->>DB: Revoke all refresh_tokens<br/>EXCEPT current session

    Note over Auth: Other sessions invalidated.<br/>User stays logged in on current device.

    Auth-->>API: Success
    API-->>SPA: 204 No Content

    Note over DB: Audit log entry created:<br/>password_changed, user_id, timestamp
```

---

## 11. Security Controls

### Request Authentication Middleware

Every protected API request passes through the auth middleware:

```mermaid
flowchart TD
    REQ["Incoming Request"] --> CHECK_HEADER{"Authorization<br/>header present?"}
    
    CHECK_HEADER -->|No| UNAUTH["401 Unauthorized"]
    CHECK_HEADER -->|Yes| EXTRACT["Extract Bearer token"]
    
    EXTRACT --> VERIFY{"Verify JWT signature<br/>(JWT_SECRET)"}
    
    VERIFY -->|Invalid| TRY_OLD{"JWT_SECRET_OLD<br/>configured?"}
    TRY_OLD -->|No| UNAUTH
    TRY_OLD -->|Yes| VERIFY_OLD{"Verify with<br/>JWT_SECRET_OLD"}
    VERIFY_OLD -->|Invalid| UNAUTH
    VERIFY_OLD -->|Valid| CHECK_EXP
    
    VERIFY -->|Valid| CHECK_EXP{"exp > now?"}
    CHECK_EXP -->|No| EXPIRED["401 token_expired"]
    CHECK_EXP -->|Yes| EXTRACT_CLAIMS["Extract sub, role"]
    
    EXTRACT_CLAIMS --> CHECK_CSRF{"X-Requested-With:<br/>RustVault?"}
    CHECK_CSRF -->|No| CSRF_ERR["403 csrf_validation_failed"]
    CHECK_CSRF -->|Yes| SET_STATE["Set AuthUser in<br/>request extensions"]
    
    SET_STATE --> HANDLER["Route Handler<br/>(user_id available)"]

    style UNAUTH fill:#fce8e8,stroke:#d9534f
    style EXPIRED fill:#fce8e8,stroke:#d9534f
    style CSRF_ERR fill:#fce8e8,stroke:#d9534f
    style HANDLER fill:#e8f8e8,stroke:#5cb85c
```

### CSRF Protection Layers

```
Layer 1: SameSite=Strict cookie
  └── Browser won't send refresh_token cookie on cross-site requests

Layer 2: X-Requested-With: RustVault header
  └── Simple CORS requests can't add custom headers
  └── Preflight required, which checks CORS origin allowlist

Layer 3: Authorization: Bearer header for API requests
  └── Not a cookie — inherently not sent automatically by browser
```

### Rate Limiting Tiers

```mermaid
graph TD
    subgraph Global["Global Rate Limit"]
        G["100 req/min/IP"]
    end

    subgraph Auth["Auth Endpoints"]
        A1["Login: 5 req/15min/IP"]
        A2["Register: 5 req/15min/IP"]
        A3["Password Reset: 3 req/hour/IP"]
    end

    subgraph Import["Import Endpoints"]
        I["50 imports/user/hour"]
    end

    subgraph Reports["Report Endpoints"]
        R["20 req/min/user"]
    end

    REQ["Request"] --> Global --> Auth & Import & Reports

    style Global fill:#e8f4fd,stroke:#4a90d9
    style Auth fill:#fce8e8,stroke:#d9534f
    style Import fill:#fff8e1,stroke:#f0ad4e
    style Reports fill:#e8f8e8,stroke:#5cb85c
```

---

## 12. Mobile Authentication

### Capacitor App Flow

```mermaid
sequenceDiagram
    actor User
    participant App as RustVault App<br/>(Capacitor)
    participant Bio as Biometric Plugin<br/>(Face ID / Fingerprint)
    participant Keychain as iOS Keychain /<br/>Android Keystore
    participant API as RustVault API

    Note over App: First login — same as web flow
    User->>App: Enter email + password
    App->>API: POST /api/auth/login
    API-->>App: access_token + refresh_token

    App->>Keychain: Store refresh_token<br/>(encrypted, hardware-backed)
    App->>App: Store access_token in memory

    Note over App: Subsequent app opens
    User->>App: Open app
    App->>Bio: Request biometric verification
    Bio-->>App: Verified ✓

    App->>Keychain: Retrieve refresh_token
    App->>API: POST /api/auth/refresh<br/>(token from keychain)
    API-->>App: New access_token + refresh_token

    App->>Keychain: Store new refresh_token
    App->>App: Store access_token in memory
    App->>App: Show dashboard

    Note over Keychain: Tokens never stored in:<br/>- SharedPreferences<br/>- NSUserDefaults<br/>- localStorage<br/>- File system
```

---

## 13. Token Storage Model

### Where Tokens Live (by Platform)

| Token | Web (SPA) | iOS (Capacitor) | Android (Capacitor) |
|-------|-----------|-----------------|---------------------|
| **Access token** | JavaScript variable | JavaScript variable | JavaScript variable |
| **Refresh token** | `HttpOnly; Secure; SameSite=Strict` cookie | iOS Keychain (via Capacitor Preferences + Biometric plugin) | Android Keystore (via Capacitor Preferences + Biometric plugin) |

### Why NOT localStorage

| Risk | localStorage | Memory (JS variable) |
|------|-------------|---------------------|
| XSS access | Readable by any JS on the page | Not accessible via DOM APIs |
| Persistence after tab close | Yes — persists until cleared | No — gone on page close |
| Accessible by browser extensions | Yes | Limited |
| Survives page refresh | Yes | No (requires refresh token to get new access token) |

The trade-off: losing the access token on page refresh requires a transparent refresh call. This is handled automatically by the SPA's HTTP client interceptor.

### SPA Token Refresh Interceptor

```
┌─────────────────────────────────────────────────┐
│ HTTP Client (fetch wrapper)                     │
│                                                 │
│  1. Attach Authorization: Bearer <access_token> │
│  2. Send request                                │
│  3. If 401 received:                            │
│     a. Call POST /api/auth/refresh              │
│     b. If refresh succeeds:                     │
│        - Update access_token in memory          │
│        - Retry original request                 │
│     c. If refresh fails:                        │
│        - Clear all auth state                   │
│        - Redirect to /login                     │
│  4. Queue concurrent requests during refresh    │
│     (only one refresh call at a time)           │
└─────────────────────────────────────────────────┘
```

---

## References

- [ADR-0008: Auth & JWT Design](../adr/0008-auth-jwt-design.md) — Decision record on JWT vs session cookies
- [RustVault API — Authentication Section](../../API_PLAN.md#2-authentication--sessions) — Full endpoint specifications
- [OWASP JWT Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html)
- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [Auth0: Refresh Token Rotation](https://auth0.com/docs/secure/tokens/refresh-tokens/refresh-token-rotation)
- [RustVault Threat Model](threat-model.md) — Threat analysis including auth-related threats (S1–S6)
