//! JWT authentication middleware.
//!
//! Extracts and validates the JWT from the `Authorization: Bearer <token>` header.
//! On success, stores [`AuthUser`] in request extensions for downstream extractors.

use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::response::ApiError;
use crate::state::AppState;

/// Axum middleware that validates JWT access tokens.
///
/// Applied to protected route groups. Skipped for public routes
/// (login, register, health, OIDC, docs).
///
/// After verifying the JWT signature and expiry, performs a lightweight
/// DB check to confirm the user still exists and reads the current role
/// (so role changes take effect immediately, not after token expiry).
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing Bearer token".into()))?;

    let claims = rustvault_core::crypto::decode_access_token(
        token,
        &state.config.jwt_secret,
        state.config.jwt_secret_old.as_deref(),
    )
    .map_err(|e| match e {
        rustvault_core::CoreError::TokenExpired => ApiError::Unauthorized("Token expired".into()),
        rustvault_core::CoreError::TokenInvalid(msg) => {
            ApiError::Unauthorized(format!("Invalid token: {msg}"))
        }
        _ => ApiError::Unauthorized("Authentication failed".into()),
    })?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Unauthorized("Invalid token subject".into()))?;

    // Validate that the user still exists and fetch current role from DB.
    // This catches deleted users immediately and ensures role changes
    // take effect without waiting for token expiry.
    let role = rustvault_db::repos::user::get_role(&state.pool, user_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => {
                ApiError::Unauthorized("User no longer exists".into())
            }
            _ => ApiError::Internal("Session validation failed".into()),
        })?;

    req.extensions_mut().insert(AuthUser { user_id, role });

    Ok(next.run(req).await)
}
