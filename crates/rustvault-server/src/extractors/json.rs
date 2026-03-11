//! Validated JSON body extractor.
//!
//! Wraps [`axum::Json`] with automatic validation via the [`validator`] crate.

use axum::extract::FromRequest;
use axum::extract::rejection::JsonRejection;
use axum::http::Request;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::response::{ApiError, FieldError};

/// Extract and validate a JSON request body.
///
/// Combines deserialization and validation in one step. If validation fails,
/// returns a structured 400 error with field-level details.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiError;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|e| ApiError::BadRequest(e.body_text()))?;

        value.validate().map_err(|errors| {
            let field_errors: Vec<FieldError> = errors
                .field_errors()
                .into_iter()
                .flat_map(|(field, errs)| {
                    errs.iter().map(move |e| FieldError {
                        field: field.to_string(),
                        message: e
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| format!("invalid value for {field}")),
                    })
                })
                .collect();
            ApiError::Validation(field_errors)
        })?;

        Ok(ValidatedJson(value))
    }
}
