//! Unified API response and error types.
//!
//! All route handlers return `Result<impl IntoResponse, ApiError>`.
//! This module maps domain errors from every crate into consistent
//! JSON error responses with proper HTTP status codes.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rustvault_core::CoreError;
use serde::Serialize;
use utoipa::ToSchema;

// ── Success wrappers ──────────────────────────────────────────

/// Standard API success response wrapping a single resource.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    /// Response payload.
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    /// Wrap data in a success response.
    pub fn ok(data: T) -> axum::Json<Self> {
        axum::Json(Self { data })
    }
}

/// Paginated API response wrapping a collection.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    /// Collection of items.
    pub data: Vec<T>,
    /// Pagination metadata.
    pub meta: PaginationMeta,
}

/// Pagination metadata.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    /// Number of items in this page.
    pub page_size: i64,
    /// Opaque cursor for the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Whether more items exist beyond this page.
    pub has_more: bool,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Wrap a collection of items with pagination metadata.
    pub fn build(data: Vec<T>, has_more: bool, next_cursor: Option<String>) -> axum::Json<Self> {
        let page_size = data.len() as i64;
        axum::Json(Self {
            data,
            meta: PaginationMeta {
                page_size,
                next_cursor,
                has_more,
            },
        })
    }

    /// Wrap a complete (non-paginated) collection as a single page.
    pub fn from_vec(data: Vec<T>) -> axum::Json<Self> {
        let page_size = data.len() as i64;
        axum::Json(Self {
            data,
            meta: PaginationMeta {
                page_size,
                next_cursor: None,
                has_more: false,
            },
        })
    }
}

// ── Error types ───────────────────────────────────────────────

/// Standard API error response body.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    /// Error details.
    pub error: ErrorData,
}

/// Error detail payload.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorData {
    /// Machine-readable error code (e.g., `"VALIDATION_ERROR"`).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional field-level validation errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

/// A single field-level validation error.
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldError {
    /// Field name.
    pub field: String,
    /// Validation message.
    pub message: String,
}

/// Unified error type for all API handlers.
#[derive(Debug)]
pub enum ApiError {
    /// 400 — Validation errors with field-level details.
    Validation(Vec<FieldError>),
    /// 400 — Generic bad request.
    BadRequest(String),
    /// 401 — Authentication required or failed.
    Unauthorized(String),
    /// 403 — Forbidden.
    Forbidden,
    /// 404 — Resource not found.
    NotFound(String),
    /// 409 — Conflict (duplicate, etc.).
    Conflict(String),
    /// 429 — Rate limited.
    RateLimited,
    /// 500 — Internal server error.
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            Self::Validation(fields) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "One or more fields are invalid".to_string(),
                Some(fields),
            ),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg, None),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg, None),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Access denied".to_string(),
                None,
            ),
            Self::NotFound(entity) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("{entity} not found"),
                None,
            ),
            Self::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg, None),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Too many requests — try again later".to_string(),
                None,
            ),
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg,
                None,
            ),
        };

        let body = ErrorBody {
            error: ErrorData {
                code: code.to_string(),
                message,
                details,
            },
        };

        (status, axum::Json(body)).into_response()
    }
}

// ── Conversions from domain errors ────────────────────────────

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::InvalidCredentials | CoreError::AuthFailed { .. } => {
                Self::Unauthorized("Invalid credentials".into())
            }
            CoreError::TokenExpired | CoreError::TokenInvalid(_) => {
                Self::Unauthorized("Token expired or invalid".into())
            }
            CoreError::AccessDenied => Self::Forbidden,
            CoreError::AccountLocked => {
                Self::Unauthorized("Account locked — too many failed attempts".into())
            }
            CoreError::NotFound { entity, .. } => Self::NotFound(entity),
            CoreError::Validation(msg) => Self::BadRequest(msg),
            CoreError::Conflict(msg) => Self::Conflict(msg),
            CoreError::Forbidden(msg) => {
                tracing::warn!(%msg, "forbidden");
                Self::Forbidden
            }
            CoreError::OidcNotConfigured => {
                Self::BadRequest("OIDC is not configured on this instance".into())
            }
            CoreError::OidcUserNotRegistered { .. } => Self::Forbidden,
            CoreError::RegistrationDisabled => Self::Forbidden,
            CoreError::PasswordLoginDisabled => {
                Self::BadRequest("This account uses SSO — sign in with your identity provider".into())
            }
            CoreError::OidcError(msg) => {
                tracing::error!(%msg, "OIDC error");
                Self::Unauthorized("SSO authentication failed".into())
            }
            CoreError::Db(db_err) => {
                tracing::error!(?db_err, "database error");
                Self::Internal("Internal server error".into())
            }
            CoreError::Internal(msg) => {
                tracing::error!(%msg, "internal error");
                Self::Internal("Internal server error".into())
            }
        }
    }
}
