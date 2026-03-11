//! Authentication service — register, login, refresh, token management.

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::crypto;
use crate::error::CoreError;
use crate::models::session::AuthTokens;
use crate::models::user::UserInfo;

/// Register a new user with local credentials.
pub async fn register(
    pool: &PgPool,
    username: &str,
    email: &str,
    password: &str,
    registration_enabled: bool,
) -> Result<UserInfo, CoreError> {
    if !registration_enabled {
        return Err(CoreError::RegistrationDisabled);
    }

    // First user ever registered gets the admin role
    let user_count = rustvault_db::repos::user::count(pool).await?;
    let role = if user_count == 0 { "admin" } else { "member" };

    let password_hash = crypto::hash_password(password)?;
    let row = rustvault_db::repos::user::insert(pool, username, email, &password_hash, role)
        .await
        .map_err(|e| match &e {
            rustvault_db::DbError::UniqueViolation(field) => {
                CoreError::Conflict(format!("{field} already exists"))
            }
            _ => CoreError::Db(e),
        })?;

    Ok(user_row_to_info(row))
}

/// Authenticate with email + password. Returns JWT tokens.
#[allow(clippy::too_many_arguments)]
pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
    access_ttl: u64,
    refresh_ttl: u64,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
) -> Result<(AuthTokens, String), CoreError> {
    let row = rustvault_db::repos::user::find_by_email(pool, email)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::InvalidCredentials,
            other => CoreError::Db(other),
        })?;

    // OIDC-only users cannot use password login
    if row.auth_provider == "oidc" {
        return Err(CoreError::PasswordLoginDisabled);
    }

    let password_hash = row
        .password_hash
        .as_deref()
        .ok_or(CoreError::PasswordLoginDisabled)?;

    if !crypto::verify_password(password, password_hash)? {
        return Err(CoreError::InvalidCredentials);
    }

    let access_token =
        crypto::encode_access_token(&row.id.to_string(), &row.role, jwt_secret, access_ttl)?;

    let refresh_token = crypto::generate_refresh_token();
    let refresh_hash = crypto::hash_refresh_token(&refresh_token);

    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(refresh_ttl as i64);
    rustvault_db::repos::session::insert(
        pool,
        row.id,
        &refresh_hash,
        user_agent,
        ip_address,
        expires_at,
    )
    .await?;

    let tokens = AuthTokens {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: access_ttl,
    };

    Ok((tokens, refresh_token))
}

/// Refresh an access token using a valid refresh token.
pub async fn refresh(
    pool: &PgPool,
    refresh_token: &str,
    jwt_secret: &str,
    access_ttl: u64,
    refresh_ttl: u64,
) -> Result<(AuthTokens, String), CoreError> {
    let token_hash = crypto::hash_refresh_token(refresh_token);

    let session = rustvault_db::repos::session::find_valid_by_hash(pool, &token_hash)
        .await
        .map_err(|_| CoreError::TokenInvalid("invalid or expired refresh token".into()))?;

    // Revoke old session (token rotation)
    rustvault_db::repos::session::revoke(pool, session.id).await?;

    // Look up user
    let user = rustvault_db::repos::user::find_by_id(pool, session.user_id).await?;

    // Issue new tokens
    let access_token =
        crypto::encode_access_token(&user.id.to_string(), &user.role, jwt_secret, access_ttl)?;

    let new_refresh = crypto::generate_refresh_token();
    let new_refresh_hash = crypto::hash_refresh_token(&new_refresh);

    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(refresh_ttl as i64);
    rustvault_db::repos::session::insert(
        pool,
        user.id,
        &new_refresh_hash,
        session.user_agent.as_deref(),
        session.ip_address.as_deref(),
        expires_at,
    )
    .await?;

    let tokens = AuthTokens {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: access_ttl,
    };

    Ok((tokens, new_refresh))
}

/// Get current user info.
pub async fn me(pool: &PgPool, user_id: Uuid) -> Result<UserInfo, CoreError> {
    let row = rustvault_db::repos::user::find_by_id(pool, user_id).await?;
    Ok(user_row_to_info(row))
}

// ── OIDC ─────────────────────────────────────────────────────

use openidconnect::core::{CoreProviderMetadata, CoreResponseType};
use openidconnect::{
    AdditionalClaims, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, RedirectUrl, Scope, TokenResponse,
};

/// Empty additional claims (we only need standard OIDC claims).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct EmptyClaims {}
impl AdditionalClaims for EmptyClaims {}

/// Discover provider metadata and build an HTTP client for OIDC.
async fn oidc_discover(
    issuer_url: &str,
) -> Result<(CoreProviderMetadata, reqwest::Client), CoreError> {
    let issuer = IssuerUrl::new(issuer_url.to_string())
        .map_err(|e| CoreError::OidcError(format!("invalid issuer URL: {e}")))?;

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CoreError::OidcError(format!("HTTP client error: {e}")))?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
        .await
        .map_err(|e| CoreError::OidcError(format!("OIDC discovery failed: {e}")))?;

    Ok((provider_metadata, http_client))
}

/// Generate the OIDC authorization URL (step 1 of the flow).
///
/// Returns `(authorize_url, csrf_state, nonce)`.
pub async fn oidc_authorize_url(
    issuer_url: &str,
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    scopes: &[String],
) -> Result<(String, String, String), CoreError> {
    let (provider_metadata, _http_client) = oidc_discover(issuer_url).await?;

    let redirect = RedirectUrl::new(redirect_url.to_string())
        .map_err(|e| CoreError::OidcError(format!("invalid redirect URL: {e}")))?;

    let client = openidconnect::core::CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
    )
    .set_redirect_uri(redirect);

    let mut auth_request = client.authorize_url(
        AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    );

    for scope in scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.clone()));
    }

    let (auth_url, csrf_state, nonce) = auth_request.url();

    Ok((
        auth_url.to_string(),
        csrf_state.secret().clone(),
        nonce.secret().clone(),
    ))
}

/// Handle the OIDC callback (step 2): exchange code for tokens, find or create user.
///
/// Returns JWT tokens + refresh token, same as local login.
#[allow(clippy::too_many_arguments)]
pub async fn oidc_callback(
    pool: &PgPool,
    issuer_url: &str,
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    code: &str,
    nonce: &str,
    auto_register: bool,
    jwt_secret: &str,
    access_ttl: u64,
    refresh_ttl: u64,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
) -> Result<(AuthTokens, String), CoreError> {
    let (provider_metadata, http_client) = oidc_discover(issuer_url).await?;

    let redirect = RedirectUrl::new(redirect_url.to_string())
        .map_err(|e| CoreError::OidcError(format!("invalid redirect URL: {e}")))?;

    let client = openidconnect::core::CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
    )
    .set_redirect_uri(redirect);

    // Exchange authorization code for tokens
    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .map_err(|e| CoreError::OidcError(format!("token exchange failed: {e}")))?
        .request_async(&http_client)
        .await
        .map_err(|e| CoreError::OidcError(format!("token exchange failed: {e}")))?;

    // Verify and extract ID token claims
    let id_token = token_response
        .id_token()
        .ok_or_else(|| CoreError::OidcError("no ID token in response".into()))?;

    let verifier = client.id_token_verifier();
    let nonce_val = Nonce::new(nonce.to_string());

    let claims = id_token
        .claims(&verifier, &nonce_val)
        .map_err(|e| CoreError::OidcError(format!("ID token verification failed: {e}")))?;

    let subject = claims.subject().to_string();
    let email: String = claims
        .email()
        .map(|e: &openidconnect::EndUserEmail| e.to_string())
        .ok_or_else(|| {
            CoreError::OidcError("OIDC provider did not return an email claim".into())
        })?;

    let username = claims
        .preferred_username()
        .map(|u: &openidconnect::EndUserUsername| u.to_string())
        .or_else(|| {
            claims
                .name()
                .and_then(
                    |n: &openidconnect::LocalizedClaim<openidconnect::EndUserName>| n.get(None),
                )
                .map(|n: &openidconnect::EndUserName| n.to_string())
        })
        .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string());

    // Find existing user by OIDC identity
    let user = rustvault_db::repos::user::find_by_oidc(pool, issuer_url, &subject).await?;

    let user_row = match user {
        Some(row) => row,
        None => {
            // Try to find by email and link OIDC identity
            if let Ok(existing) = rustvault_db::repos::user::find_by_email(pool, &email).await {
                rustvault_db::repos::user::link_oidc(pool, existing.id, issuer_url, &subject)
                    .await?;
                rustvault_db::repos::user::find_by_id(pool, existing.id).await?
            } else if auto_register {
                // Auto-provision new OIDC user
                rustvault_db::repos::user::insert_oidc(
                    pool, &username, &email, issuer_url, &subject,
                )
                .await
                .map_err(|e| match &e {
                    rustvault_db::DbError::UniqueViolation(_) => {
                        CoreError::Conflict("user already exists".into())
                    }
                    _ => CoreError::Db(e),
                })?
            } else {
                return Err(CoreError::OidcUserNotRegistered { email });
            }
        }
    };

    // Issue JWT tokens (same as local login)
    let access_token = crypto::encode_access_token(
        &user_row.id.to_string(),
        &user_row.role,
        jwt_secret,
        access_ttl,
    )?;

    let refresh_token = crypto::generate_refresh_token();
    let refresh_hash = crypto::hash_refresh_token(&refresh_token);

    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(refresh_ttl as i64);
    rustvault_db::repos::session::insert(
        pool,
        user_row.id,
        &refresh_hash,
        user_agent,
        ip_address,
        expires_at,
    )
    .await?;

    let tokens = AuthTokens {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: access_ttl,
    };

    Ok((tokens, refresh_token))
}

/// Convert a database user row to public user info.
fn user_row_to_info(row: rustvault_db::repos::user::UserRow) -> UserInfo {
    UserInfo {
        id: row.id,
        username: row.username,
        email: row.email,
        role: match row.role.as_str() {
            "admin" => crate::models::user::UserRole::Admin,
            "member" => crate::models::user::UserRole::Member,
            "viewer" => crate::models::user::UserRole::Viewer,
            _ => crate::models::user::UserRole::Member,
        },
        auth_provider: match row.auth_provider.as_str() {
            "local" => crate::models::user::AuthProvider::Local,
            "oidc" => crate::models::user::AuthProvider::Oidc,
            "both" => crate::models::user::AuthProvider::Both,
            _ => crate::models::user::AuthProvider::Local,
        },
        locale: row.locale,
        timezone: row.timezone,
        settings: row.settings,
        created_at: row.created_at,
    }
}
