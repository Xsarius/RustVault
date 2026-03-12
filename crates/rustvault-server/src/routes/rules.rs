//! Auto-categorization rule CRUD routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::extractors::json::ValidatedJson;
use crate::response::{ApiError, ApiResponse, ErrorBody, PaginatedResponse};
use crate::state::AppState;

use rustvault_core::models::rule::{NewAutoRule, UpdateAutoRule};
use rustvault_core::services::rule_engine;

/// `GET /api/rules` — List auto-categorization rules.
#[utoipa::path(
    get,
    path = "/api/rules",
    tag = "Rules",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "List of rules", body = inline(PaginatedResponse<rustvault_core::models::rule::AutoRule>)),
        (status = 401, description = "Not authenticated", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let rules = rustvault_core::services::rule::list(&state.pool, auth.user_id).await?;
    Ok(PaginatedResponse::from_vec(rules))
}

/// `POST /api/rules` — Create a new auto-categorization rule.
#[utoipa::path(
    post,
    path = "/api/rules",
    tag = "Rules",
    security(("bearer" = [])),
    request_body = NewAutoRule,
    responses(
        (status = 201, description = "Rule created", body = inline(ApiResponse<rustvault_core::models::rule::AutoRule>)),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(body): ValidatedJson<NewAutoRule>,
) -> Result<impl IntoResponse, ApiError> {
    let rule = rustvault_core::services::rule::create(
        &state.pool,
        auth.user_id,
        &body.name,
        body.priority.unwrap_or(0),
        &body.conditions,
        &body.actions,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(rule)))
}

/// `GET /api/rules/:id` — Get a single rule.
#[utoipa::path(
    get,
    path = "/api/rules/{id}",
    tag = "Rules",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Rule ID")),
    responses(
        (status = 200, description = "Rule details", body = inline(ApiResponse<rustvault_core::models::rule::AutoRule>)),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let rule = rustvault_core::services::rule::get(&state.pool, auth.user_id, id).await?;
    Ok(ApiResponse::ok(rule))
}

/// `PUT /api/rules/:id` — Update a rule.
#[utoipa::path(
    put,
    path = "/api/rules/{id}",
    tag = "Rules",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Rule ID")),
    request_body = UpdateAutoRule,
    responses(
        (status = 200, description = "Rule updated", body = inline(ApiResponse<rustvault_core::models::rule::AutoRule>)),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateAutoRule>,
) -> Result<impl IntoResponse, ApiError> {
    let rule = rustvault_core::services::rule::update(
        &state.pool,
        auth.user_id,
        id,
        body.name.as_deref(),
        body.priority,
        body.is_enabled,
        body.conditions.as_ref(),
        body.actions.as_ref(),
    )
    .await?;
    Ok(ApiResponse::ok(rule))
}

/// `DELETE /api/rules/:id` — Delete a rule.
#[utoipa::path(
    delete,
    path = "/api/rules/{id}",
    tag = "Rules",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Rule ID")),
    responses(
        (status = 204, description = "Rule deleted"),
        (status = 404, description = "Not found", body = ErrorBody),
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    rustvault_core::services::rule::delete(&state.pool, auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Test / Suggest endpoints ───────────────────────────────────

/// Request body for testing a rule against a sample transaction.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct TestRuleRequest {
    /// Conditions to test (JSONB array).
    pub conditions: serde_json::Value,
    /// Transaction description to test against.
    pub description: String,
    /// Transaction payee to test against.
    pub payee: Option<String>,
    /// Transaction amount.
    pub amount: Decimal,
    /// Account ID.
    pub account_id: Uuid,
}

/// Response for rule-test endpoint.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TestRuleResponse {
    /// Whether the rule matched.
    pub matched: bool,
}

/// `POST /api/rules/test` — Test a rule's conditions against a sample transaction.
#[utoipa::path(
    post,
    path = "/api/rules/test",
    tag = "Rules",
    security(("bearer" = [])),
    request_body = TestRuleRequest,
    responses(
        (status = 200, description = "Test result", body = inline(ApiResponse<TestRuleResponse>)),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
)]
pub async fn test_rule(
    State(_state): State<AppState>,
    _auth: AuthUser,
    ValidatedJson(body): ValidatedJson<TestRuleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Build a temporary AutoRule for evaluation.
    let temp_rule = rustvault_core::models::rule::AutoRule {
        id: Uuid::nil(),
        user_id: Uuid::nil(),
        name: "test".into(),
        priority: 0,
        is_enabled: true,
        conditions: body.conditions,
        actions: serde_json::json!([]),
        metadata: serde_json::json!({}),
        created_at: time::OffsetDateTime::now_utc(),
        updated_at: time::OffsetDateTime::now_utc(),
    };

    let candidate = rule_engine::MatchCandidate {
        description: body.description,
        original_desc: None,
        payee: body.payee,
        amount: body.amount,
        account_id: body.account_id,
    };

    let matched = rule_engine::evaluate_rule(&temp_rule, &candidate);
    Ok(ApiResponse::ok(TestRuleResponse { matched }))
}

/// Request body for rule suggestion.
#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct SuggestRuleRequest {
    /// Transaction description from which to derive a rule.
    pub description: String,
    /// Transaction payee.
    pub payee: Option<String>,
    /// Transaction amount.
    pub amount: Decimal,
}

/// Response for rule-suggest endpoint.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SuggestRuleResponse {
    /// Suggested rule name.
    pub name: String,
    /// Suggested conditions.
    pub conditions: Vec<rule_engine::RuleCondition>,
}

/// `POST /api/rules/suggest` — Suggest a rule based on a transaction.
#[utoipa::path(
    post,
    path = "/api/rules/suggest",
    tag = "Rules",
    security(("bearer" = [])),
    request_body = SuggestRuleRequest,
    responses(
        (status = 200, description = "Suggested rule", body = inline(ApiResponse<SuggestRuleResponse>)),
        (status = 400, description = "Validation error", body = ErrorBody),
    ),
)]
pub async fn suggest(
    State(_state): State<AppState>,
    _auth: AuthUser,
    ValidatedJson(body): ValidatedJson<SuggestRuleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let (name, conditions) =
        rule_engine::suggest_rule(&body.description, body.payee.as_deref(), body.amount);
    Ok(ApiResponse::ok(SuggestRuleResponse { name, conditions }))
}
