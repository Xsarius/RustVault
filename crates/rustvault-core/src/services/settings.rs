//! Settings service — get and update user preferences.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::CoreError;
use crate::models::settings::{UpdateSettings, UserSettings};

/// Helper to read a string from a JSONB value with a default.
fn json_str(v: &serde_json::Value, key: &str, default: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_owned()
}

/// Helper to read an optional string from a JSONB value.
fn json_opt_str(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|v| v.as_str()).map(str::to_owned)
}

/// Helper to read a bool from a JSONB value with a default.
fn json_bool(v: &serde_json::Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Helper to read a f64 from a JSONB value with a default.
fn json_f64(v: &serde_json::Value, key: &str, default: f64) -> f64 {
    v.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

/// Build a [`UserSettings`] from the user row's columns + JSONB settings blob.
fn row_to_settings(row: &rustvault_db::repos::user::UserRow) -> UserSettings {
    let s = &row.settings;
    UserSettings {
        locale: row.locale.clone(),
        timezone: row.timezone.clone(),
        default_currency: json_str(s, "default_currency", "USD"),
        date_format: json_str(s, "date_format", "YYYY-MM-DD"),
        theme: json_str(s, "theme", "system"),
        ai_enabled: json_bool(s, "ai_enabled", false),
        ai_provider: json_opt_str(s, "ai_provider"),
        ai_model_text: json_opt_str(s, "ai_model_text"),
        ai_model_vision: json_opt_str(s, "ai_model_vision"),
        ai_confidence_threshold: json_f64(s, "ai_confidence_threshold", 0.7),
        ai_receipt_scanning: json_bool(s, "ai_receipt_scanning", true),
        ai_categorization_suggestions: json_bool(s, "ai_categorization_suggestions", true),
        ai_import_enrichment: json_bool(s, "ai_import_enrichment", false),
        ai_payee_normalization: json_bool(s, "ai_payee_normalization", true),
    }
}

/// Get the current settings for a user.
pub async fn get(pool: &PgPool, user_id: Uuid) -> Result<UserSettings, CoreError> {
    let row = rustvault_db::repos::user::find_by_id(pool, user_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "user".into(),
                id: user_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;
    Ok(row_to_settings(&row))
}

/// Update user settings (partial update — only provided fields change).
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    input: &UpdateSettings,
) -> Result<UserSettings, CoreError> {
    // First fetch the current user to get the existing JSONB settings blob.
    let current = rustvault_db::repos::user::find_by_id(pool, user_id)
        .await
        .map_err(|e| match e {
            rustvault_db::DbError::NotFound => CoreError::NotFound {
                entity: "user".into(),
                id: user_id.to_string(),
            },
            other => CoreError::Db(other),
        })?;

    // Merge JSONB settings: start from existing, overlay provided fields.
    let mut s = current.settings.clone();
    if !s.is_object() {
        s = serde_json::Value::Object(serde_json::Map::new());
    }
    let obj = s.as_object_mut().expect("settings is always an object");

    if let Some(ref v) = input.default_currency {
        obj.insert("default_currency".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(ref v) = input.date_format {
        obj.insert("date_format".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(ref v) = input.theme {
        obj.insert("theme".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(v) = input.ai_enabled {
        obj.insert("ai_enabled".into(), serde_json::Value::Bool(v));
    }
    if let Some(ref v) = input.ai_provider {
        obj.insert("ai_provider".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(ref v) = input.ai_model_text {
        obj.insert("ai_model_text".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(ref v) = input.ai_model_vision {
        obj.insert("ai_model_vision".into(), serde_json::Value::String(v.clone()));
    }
    if let Some(v) = input.ai_confidence_threshold {
        obj.insert(
            "ai_confidence_threshold".into(),
            serde_json::json!(v),
        );
    }
    if let Some(v) = input.ai_receipt_scanning {
        obj.insert("ai_receipt_scanning".into(), serde_json::Value::Bool(v));
    }
    if let Some(v) = input.ai_categorization_suggestions {
        obj.insert(
            "ai_categorization_suggestions".into(),
            serde_json::Value::Bool(v),
        );
    }
    if let Some(v) = input.ai_import_enrichment {
        obj.insert("ai_import_enrichment".into(), serde_json::Value::Bool(v));
    }
    if let Some(v) = input.ai_payee_normalization {
        obj.insert("ai_payee_normalization".into(), serde_json::Value::Bool(v));
    }

    // `s` was mutated in-place via `obj`; pass it directly.
    // Persist via the existing repo function.
    let row = rustvault_db::repos::user::update_settings(
        pool,
        user_id,
        input.locale.as_deref(),
        input.timezone.as_deref(),
        Some(&s),
    )
    .await
    .map_err(|e| match e {
        rustvault_db::DbError::NotFound => CoreError::NotFound {
            entity: "user".into(),
            id: user_id.to_string(),
        },
        other => CoreError::Db(other),
    })?;

    // Audit log.
    let old_value = serde_json::to_value(&row_to_settings(&current)).ok();
    let new_value = serde_json::to_value(&row_to_settings(&row)).ok();
    let _ = rustvault_db::repos::audit::insert(
        pool,
        user_id,
        "settings",
        user_id,
        "update",
        old_value.as_ref(),
        new_value.as_ref(),
    )
    .await;

    Ok(row_to_settings(&row))
}
