//! User settings model.
//!
//! Settings combine top-level user columns (`locale`, `timezone`) with
//! the JSONB `settings` field that stores extended preferences.

use serde::{Deserialize, Serialize};

/// Full user settings returned by `GET /api/settings`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UserSettings {
    /// Preferred locale (BCP 47), e.g. `"en-US"`.
    pub locale: String,
    /// Preferred timezone (IANA), e.g. `"America/New_York"`.
    pub timezone: String,
    /// Default currency for new accounts / reports (ISO 4217).
    pub default_currency: String,
    /// Preferred date format (e.g. `"YYYY-MM-DD"`, `"MM/DD/YYYY"`).
    pub date_format: String,
    /// UI theme preference.
    pub theme: String,
    /// Whether AI features are enabled.
    pub ai_enabled: bool,
    /// Active AI provider (e.g. `"ollama"`, `"openai"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_provider: Option<String>,
    /// AI text model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model_text: Option<String>,
    /// AI vision model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model_vision: Option<String>,
    /// Minimum confidence for auto-applying AI suggestions.
    pub ai_confidence_threshold: f64,
    /// Enable AI receipt scanning.
    pub ai_receipt_scanning: bool,
    /// Enable AI categorization suggestions.
    pub ai_categorization_suggestions: bool,
    /// Enable AI enrichment during import.
    pub ai_import_enrichment: bool,
    /// Enable AI payee name normalization.
    pub ai_payee_normalization: bool,
}

/// Request body for `PUT /api/settings` (partial update).
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct UpdateSettings {
    /// Preferred locale.
    #[validate(length(min = 2, max = 35))]
    pub locale: Option<String>,
    /// Preferred timezone.
    #[validate(length(min = 1, max = 64))]
    pub timezone: Option<String>,
    /// Default currency (ISO 4217).
    #[validate(length(min = 3, max = 3))]
    pub default_currency: Option<String>,
    /// Preferred date format.
    #[validate(length(min = 1, max = 20))]
    pub date_format: Option<String>,
    /// UI theme.
    #[validate(length(min = 1, max = 20))]
    pub theme: Option<String>,
    /// AI master toggle.
    pub ai_enabled: Option<bool>,
    /// AI provider.
    #[validate(length(min = 1, max = 50))]
    pub ai_provider: Option<String>,
    /// AI text model.
    #[validate(length(min = 1, max = 100))]
    pub ai_model_text: Option<String>,
    /// AI vision model.
    #[validate(length(min = 1, max = 100))]
    pub ai_model_vision: Option<String>,
    /// AI confidence threshold.
    pub ai_confidence_threshold: Option<f64>,
    /// AI receipt scanning toggle.
    pub ai_receipt_scanning: Option<bool>,
    /// AI categorization suggestions toggle.
    pub ai_categorization_suggestions: Option<bool>,
    /// AI import enrichment toggle.
    pub ai_import_enrichment: Option<bool>,
    /// AI payee normalization toggle.
    pub ai_payee_normalization: Option<bool>,
}
