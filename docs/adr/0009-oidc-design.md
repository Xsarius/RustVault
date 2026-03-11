# ADR-0009: OIDC Integration Design

- **Status:** Accepted
- **Date:** 2026-03-15
- **Deciders:** RustVault core team
- **Tags:** backend, security, auth, oidc, sso

## Context

RustVault targets self-hosters who often run centralised identity management (Authentik, Keycloak, Authelia). Users expect to sign in with their existing SSO credentials rather than managing a separate password for every self-hosted application.

The key requirements:

- Support any standards-compliant **OpenID Connect 1.0** provider
- Integrate cleanly with the existing JWT-based auth system (see [ADR-0008](0008-auth-jwt-design.md))
- Allow both OIDC-only and password+OIDC hybrid deployments
- Minimise attack surface — no long-lived provider tokens stored server-side
- Work without additional infrastructure (no Redis, no session store)

## Decision

### Authorization Code Flow with PKCE-Ready Design

RustVault implements the standard **OIDC Authorization Code** flow:

1. **`GET /api/auth/oidc/authorize`** — generates the authorization URL with `state` (CSRF) and `nonce` (replay) parameters. Returns the URL plus the state/nonce values for the client to store.
2. Client redirects the user to the OIDC provider.
3. Provider redirects back with an authorization `code`.
4. **`POST /api/auth/oidc/callback`** — exchanges the code for tokens, validates the ID token (signature + nonce), extracts user claims, and issues RustVault's own JWT access + refresh tokens.

### Provider Token Handling

- The OIDC **ID token** is validated and then discarded — only the `sub`, `email`, and `name` claims are extracted.
- The OIDC **access token** is not stored or forwarded. RustVault does not call provider APIs after authentication.
- The OIDC **refresh token** is never requested or stored.

This means RustVault sessions are independent of the provider session. Signing out of the OIDC provider does not revoke the RustVault session (and vice versa). This is intentional to avoid coupling availability to the provider.

### User Linking Strategy

| Scenario | Behaviour |
|----------|-----------|
| New OIDC user, matching email exists | Link the OIDC identity (`oidc_sub`) to the existing account. User can now sign in with either method. |
| New OIDC user, no matching email, `auto_register = true` | Create a new user with `auth_provider = oidc`, `oidc_sub` set to the provider's `sub` claim |
| New OIDC user, no matching email, `auto_register = false` | Return `403 Forbidden` |
| Returning OIDC user | Look up by `oidc_sub`, issue tokens |

Email-based merging is appropriate because RustVault is self-hosted — the administrator controls both the application and the OIDC provider, so the IdP is trusted as a source of truth for email ownership. This matches the behaviour of other self-hosted tools (Grafana, Gitea, Outline).

### Library Choice: `openidconnect` Crate

The `openidconnect` crate (v4) provides:

- OIDC Discovery (`.well-known/openid-configuration` fetched at authorize time)
- ID token verification (signature, issuer, audience, nonce, expiry)
- Support for RS256, ES256, and other standard signing algorithms
- Minimal runtime dependencies

Discovery is performed per-authorize request rather than cached at startup, because:
- The OIDC provider may be unavailable at server startup
- Key rotation by the provider is handled automatically
- The latency hit (~one HTTP call) only occurs during login, not per-request

### Configuration

OIDC is configured via three layers:

| Setting | Source | Rationale |
|---------|--------|-----------|
| `OIDC_CLIENT_ID` | Environment variable | Secret in some threat models |
| `OIDC_CLIENT_SECRET` | Environment variable | Always secret |
| `OIDC_ISSUER_URL` | Environment variable | May contain internal hostnames |
| `enabled`, `display_name`, `scopes`, `auto_register` | `config.toml` `[auth.oidc]` | Operational tuning, not secret |

The feature is **disabled by default** (`enabled = false`). When disabled, the authorize and callback endpoints return a clear error rather than silently failing.

## Alternatives Considered

### Option A: SAML 2.0

| Pros | Cons |
|------|------|
| Mature enterprise standard | XML-heavy, complex to implement |
| Supported by most IdPs | OIDC already covers the self-hosted IdP market |

Rejected — OIDC is simpler, better supported in the Rust ecosystem, and sufficient for the target audience.

### Option B: Store OIDC Refresh Tokens and Proxy API Calls

| Pros | Cons |
|------|------|
| Could revoke on provider signout | Requires storing sensitive provider tokens |
| Enables calling provider APIs | Increases coupling and attack surface |

Rejected — RustVault only needs identity verification, not ongoing API access. Storing provider tokens adds complexity and risk with no user-facing benefit.

### Option C: Cache OIDC Discovery at Startup

| Pros | Cons |
|------|------|
| Faster login flow | Server fails to start if provider is down |
| Single network call | Stale keys if provider rotates during runtime |

Rejected — per-request discovery is slightly slower but more resilient and operationally safer for self-hosted environments where the IdP may start after RustVault.

### Option D: Never Merge — Keep OIDC and Password Users Separate

| Pros | Cons |
|------|------|
| Eliminates theoretical account takeover via email | Terrible UX — existing users lose data when switching to SSO |
| Simpler linking logic | Forces admins to manually re-create accounts |

Rejected — in a self-hosted context the admin controls the IdP and trusts it as an email authority. The usability cost of requiring manual account linking outweighs the theoretical risk.

## Consequences

### Positive

- Any OIDC-compliant provider works out of the box
- Zero additional infrastructure (no Redis, no session store)
- Email-based account linking makes password → OIDC migration seamless
- Clean separation — OIDC is for identity, RustVault manages its own sessions
- Simple self-hoster configuration (three env vars + one config flag)
- No sensitive provider tokens stored server-side
- Disabled by default — zero attack surface increase for users who don't need SSO

### Negative

- No provider-initiated single logout (signing out of the IdP does not revoke the RustVault session)
- Cannot call provider APIs (user info, group membership) after the initial login

### Risks

- OIDC provider compromise → attacker can authenticate as any provider user
- Provider `sub` claim instability → user loses access to their RustVault account (mitigated by using `sub`, which is stable per OIDC spec)
- Provider downtime → OIDC login unavailable (password login still works if configured)

## References

- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [ADR-0008: Authentication & JWT Design](0008-auth-jwt-design.md)
- [RFC 6749: OAuth 2.0 Authorization Framework](https://datatracker.ietf.org/doc/html/rfc6749)
- [`openidconnect` crate documentation](https://docs.rs/openidconnect/latest/openidconnect/)
- [OIDC / SSO Setup Guide](../book/src/self-hosting/oidc-setup.md)
