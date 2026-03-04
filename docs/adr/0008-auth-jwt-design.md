# ADR-0008: Authentication & JWT Design

- **Status:** Accepted
- **Date:** 2026-03-03
- **Deciders:** RustVault core team
- **Tags:** backend, security, auth, jwt

## Context

RustVault handles sensitive personal financial data (transactions, account balances, bank details). The authentication system must:

- Protect against session hijacking, XSS token theft, and CSRF attacks
- Support stateless API verification (no database call per request)
- Work seamlessly across web SPA (SolidJS), iOS, and Android (Capacitor) clients
- Allow multiple concurrent sessions with per-session revocation
- Support future OIDC/SSO integration without architectural changes
- Be simple enough for self-hosters to operate (no Redis, no external session stores)

The core question: **should RustVault use server-side sessions (cookies) or JWTs for API authentication?**

## Decision

### Dual-Token Architecture: JWT Access Token + Opaque Refresh Token

RustVault uses a **split-token design**:

1. **Access token** — a short-lived JWT (15 minutes) stored in browser memory, sent via `Authorization: Bearer` header
2. **Refresh token** — a longer-lived opaque random string (7 days) stored in an `HttpOnly; Secure; SameSite=Strict` cookie, validated and rotated server-side

### JWT Signing: HMAC-SHA256 (HS256)

- Single-server deployment — no need for asymmetric keys (RS256)
- `JWT_SECRET` env var with minimum 256-bit entropy
- Key rotation via dual-key validation (`JWT_SECRET` + `JWT_SECRET_OLD`)

### Access Token Claims (Minimal)

```json
{
  "sub": "<user_id (UUID)>",
  "role": "admin | member",
  "iat": 1709463600,
  "exp": 1709464500
}
```

No email, username, or settings in the JWT — keeps tokens small and avoids stale claim issues. Profile data is fetched via `GET /api/auth/me` and cached client-side.

### Refresh Token Rotation (Single-Use)

Each refresh request:
1. Validates the presented token (hash lookup in `refresh_tokens` table)
2. Marks the old token as consumed (`consumed_at = NOW()`)
3. Issues a new refresh token and new access token
4. If a consumed token is presented again → **theft detected** → **all user sessions revoked**

### Token Storage by Platform

| Token | Web (SPA) | Mobile (Capacitor) |
|-------|-----------|-------------------|
| Access token | JavaScript variable (memory) | JavaScript variable (memory) |
| Refresh token | `HttpOnly; Secure; SameSite=Strict` cookie | iOS Keychain / Android Keystore (hardware-backed, biometric-gated) |

## Alternatives Considered

### Option A: Server-Side Sessions with Cookies

| Pros | Cons |
|------|------|
| Simple mental model — session ID in cookie, state in DB/Redis | Requires a DB or Redis lookup on **every** API request |
| Easy revocation — delete the session row | Adds latency to every request (session lookup) |
| No token parsing/validation overhead | Requires sticky sessions or shared session store for horizontal scaling |
| Naturally `HttpOnly` — no XSS exposure | CSRF is a primary concern — `SameSite` alone isn't sufficient on older browsers |
| | Doesn't work well with mobile apps (cookie handling in Capacitor is fragile) |
| | Cross-origin API calls (mobile app → server) require `SameSite=None; Secure` which weakens CSRF protection |

**Why not chosen:** RustVault serves three client types (web SPA, iOS, Android) from a single API. Cookie-based sessions have inconsistent behavior across these platforms. The per-request DB lookup adds unnecessary latency for a self-hosted single-server deployment.

### Option B: JWT-Only (No Refresh Token)

| Pros | Cons |
|------|------|
| Simplest implementation — one token, no rotation | Long-lived JWT means long exposure window if stolen |
| True statelessness | Cannot revoke individual tokens without a blocklist (negates statelessness) |
| | Short-lived JWT without refresh means frequent re-login (bad UX) |

**Why not chosen:** A single JWT must be either long-lived (insecure) or short-lived (bad UX). The dual-token approach gives both short exposure (15 min access tokens) and long sessions (7-day refresh tokens) with revocability.

### Option C: JWT in `HttpOnly` Cookie (Access + Refresh Both in Cookies)

| Pros | Cons |
|------|------|
| JWT not accessible to JavaScript (no XSS theft) | CSRF becomes the primary attack vector — must add CSRF tokens |
| Automatic transmission by browser | Cookies sent on every request including images, iframes — larger attack surface |
| | JWT in cookie can be large (header overhead on every request) |
| | Mobile apps have poor cookie support — Capacitor's cookie handling is unreliable across platforms |
| | Double-submit CSRF pattern adds complexity |

**Why not chosen:** Trading XSS protection for CSRF exposure is not a net gain. Our approach stores the access token in memory (XSS-resistant) and the refresh token in an `HttpOnly` cookie (XSS-immune), avoiding both attack vectors.

### Option D: Asymmetric JWT Signing (RS256 / EdDSA)

| Pros | Cons |
|------|------|
| Public key can verify tokens without the signing key | Unnecessary for single-server architecture |
| Useful for microservices (verify without shared secret) | Larger tokens (~3x header size vs HS256) |
| Key rotation can use JWKS endpoint | More complex key management (key pairs, certificate rotation) |
| | Slower signing/verification than HS256 |

**Why not chosen:** RustVault is a monolithic single-binary deployment. There's no separate service that needs to verify tokens without access to the signing key. HS256 is simpler, faster, and sufficient. If RustVault ever needs microservice decomposition, this decision can be revisited (forward-compatible — just switch the algorithm and publish a JWKS endpoint).

### Option E: Paseto (Platform-Agnostic Security Tokens)

| Pros | Cons |
|------|------|
| Removes algorithm confusion attacks by design | Much smaller ecosystem than JWT |
| Stronger defaults (no `alg: none`, no weak algorithms) | Fewer libraries, less community knowledge |
| Versioned protocol | Not an IETF standard (JWT is RFC 7519) |
| | OIDC ecosystem is entirely JWT-based — would need two token formats |

**Why not chosen:** While Paseto has better defaults, JWT with HS256 and proper validation is equally secure. The overwhelming ecosystem support for JWT (OIDC, existing libraries, `jsonwebtoken` crate maturity) outweighs Paseto's design advantages.

## Consequences

### Positive

- **No per-request DB call** — JWT verification is pure computation (HMAC check + claim validation), keeping API latency low on self-hosted hardware
- **Cross-platform consistency** — same `Authorization: Bearer` flow works identically for web SPA, iOS, and Android
- **Revocable sessions** — refresh tokens are server-side (DB allowlist) with per-session revocation and "log out everywhere"
- **Theft detection** — single-use refresh token rotation catches replay attacks and revokes all sessions as a safety measure
- **XSS + CSRF resistant** — access token in memory (not accessible to XSS), refresh token in `HttpOnly; SameSite=Strict` cookie (not sent cross-origin), custom `X-Requested-With` header blocks CSRF
- **Zero external dependencies** — no Redis, no session store — just PostgreSQL (already required for data)
- **Simple key management** — single `JWT_SECRET` env var, optional `JWT_SECRET_OLD` for zero-downtime rotation
- **OIDC-compatible** — OIDC flow produces the same RustVault JWT — downstream API calls are unaware of the auth method

### Negative

- **15-minute revocation gap** — access tokens cannot be revoked before expiry (acceptable trade-off for stateless verification; mitigated by short TTL)
- **Refresh token DB table** — requires periodic cleanup of expired/consumed tokens (cron job or background task)
- **Page refresh loses access token** — SPA must transparently call `/api/auth/refresh` on startup (minor UX cost, but standard practice)
- **Token size overhead** — JWT is ~300 bytes per request in the `Authorization` header (negligible)
- **Clock sensitivity** — JWT expiry depends on synchronized clocks between server and client (mitigated by 60-second refresh buffer)

### Risks

- **JWT_SECRET compromise** — attacker can forge any access token. Mitigated: minimum 256-bit entropy, env var only (never in config files), key rotation support
- **Refresh token database bottleneck** — under extreme load, refresh operations could contend on the `refresh_tokens` table. Mitigated: indexed lookups, token cleanup, and the refresh endpoint is called infrequently (once per 15 minutes per session)
- **HS256 is symmetric** — if RustVault ever needs distributed token verification, migration to RS256/EdDSA will require a coordinated rollout. Mitigated: the JWT layer is isolated in the `rustvault-core` crypto module, algorithm change is localized

## References

- [RFC 7519 — JSON Web Token (JWT)](https://datatracker.ietf.org/doc/html/rfc7519)
- [RFC 6749 — OAuth 2.0 Authorization Framework](https://datatracker.ietf.org/doc/html/rfc6749)
- [Auth0: Refresh Token Rotation](https://auth0.com/docs/secure/tokens/refresh-tokens/refresh-token-rotation)
- [OWASP: JWT Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html)
- [OWASP: Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [Hasura: The Ultimate Guide to JWT Auth](https://hasura.io/blog/best-practices-of-using-jwt-with-graphql/)
- [RustVault Auth Architecture](../security/auth-architecture.md)
