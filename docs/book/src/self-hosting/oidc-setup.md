# OIDC / SSO Setup

RustVault supports **OpenID Connect (OIDC)** for single sign-on. This allows users to authenticate via an external identity provider such as [Authentik](https://goauthentik.io/), [Keycloak](https://www.keycloak.org/), [Authelia](https://www.authelia.com/), or any standards-compliant OIDC provider.

## Prerequisites

- A running OIDC provider with admin access
- RustVault accessible via HTTPS (recommended for production)
- The OIDC provider must support the **Authorization Code** flow

## How It Works

1. User clicks **"Sign in with SSO"** on the login page
2. RustVault redirects to the OIDC provider's authorization endpoint
3. User authenticates with the provider
4. Provider redirects back to RustVault with an authorization code
5. RustVault exchanges the code for an ID token, extracts the user's email and name
6. If `auto_register` is enabled, a new RustVault account is created automatically
7. RustVault issues its own JWT access token and refresh token

## Provider Setup: Authentik (Example)

### 1. Create an Application

1. In Authentik, navigate to **Applications → Applications → Create**
2. Set **Name** to `RustVault`
3. Set **Slug** to `rustvault`

### 2. Create an OAuth2/OIDC Provider

1. Go to **Applications → Providers → Create**
2. Select **OAuth2/OpenID Provider**
3. Configure:

   | Setting | Value |
   |---------|-------|
   | Name | `RustVault OIDC` |
   | Authorization flow | `default-authorization-flow` |
   | Client type | `Confidential` |
   | Redirect URIs | `https://your-domain.com/api/auth/oidc/callback` |
   | Scopes | `openid`, `profile`, `email` |

4. After saving, copy the **Client ID** and **Client Secret**

### 3. Link Provider to Application

1. Edit the `RustVault` application
2. Set **Provider** to the OIDC provider you just created

### 4. Note the Issuer URL

The issuer URL for Authentik follows this pattern:
```
https://auth.example.com/application/o/rustvault/
```

## RustVault Configuration

### Environment Variables

Set these environment variables (or add them to your `.env` file):

```bash
OIDC_ISSUER_URL=https://auth.example.com/application/o/rustvault/
OIDC_CLIENT_ID=your-client-id
OIDC_CLIENT_SECRET=your-client-secret
```

### Config File (`config.toml`)

Enable OIDC in the auth section:

```toml
[auth.oidc]
enabled = true
display_name = "Sign in with Authentik"
scopes = ["openid", "profile", "email"]
auto_register = true
```

| Key | Description |
|-----|-------------|
| `enabled` | Set to `true` to show the SSO button on the login page |
| `display_name` | Text shown on the SSO login button |
| `scopes` | OAuth scopes to request (most providers need all three) |
| `auto_register` | When `true`, first-time OIDC users get a RustVault account automatically. When `false`, an admin must pre-create the account. |

### Restart RustVault

After updating configuration, restart the service:

```bash
docker compose restart app
```

## Other Providers

The setup is similar for other OIDC providers. Key details to configure:

### Keycloak

- **Issuer URL:** `https://keycloak.example.com/realms/your-realm`
- Create a client with **Client authentication** enabled (confidential)
- Add the redirect URI: `https://your-domain.com/api/auth/oidc/callback`

### Authelia

- **Issuer URL:** `https://auth.example.com`
- Add a client entry in Authelia's `configuration.yml` under `identity_providers.oidc.clients`

### Google / Azure AD / Okta

Any provider that supports **OpenID Connect Discovery** (`.well-known/openid-configuration`) will work. Set the issuer URL to the provider's discovery base URL.

## Troubleshooting

| Symptom | Likely Cause |
|---------|-------------|
| "OIDC not configured" error | `enabled = false` in config, or missing `OIDC_*` env vars |
| Redirect loop | Incorrect redirect URI in the provider. Must exactly match `https://your-domain.com/api/auth/oidc/callback` |
| "Invalid nonce" error | Client is not storing the nonce from the authorize step. Ensure the frontend sends it back with the callback |
| User created but no data | Expected — new OIDC users start with empty accounts. See [Setting Up Accounts](../getting-started/setting-up-accounts.md) |

## Security Considerations

- Always use **HTTPS** in production for both RustVault and the OIDC provider
- Store `OIDC_CLIENT_SECRET` only in environment variables, never in config files
- The `state` parameter protects against CSRF; the `nonce` protects against replay attacks
- Consider setting `auto_register = false` and pre-creating users if you want to restrict access
