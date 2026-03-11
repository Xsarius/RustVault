//! Authenticated user extractor.
//!
//! Extracts the authenticated user from request extensions (set by auth middleware).

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::response::ApiError;
use crate::state::AppState;

/// Authenticated user info extracted from the JWT.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// User ID from the JWT `sub` claim.
    pub user_id: Uuid,
    /// User role from the JWT `role` claim.
    pub role: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| ApiError::Unauthorized("Missing or invalid authentication".into()))
    }
}
