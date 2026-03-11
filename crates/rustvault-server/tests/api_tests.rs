//! Integration tests for all P1 API endpoints.
//!
//! Uses `sqlx::test` for ephemeral databases and `axum-test` for
//! zero-overhead HTTP testing without a TCP listener.

mod helpers;

use helpers::{register_and_login, test_server, TEST_EMAIL, TEST_PASSWORD, TEST_USER};
use serde_json::{json, Value};

// ============================================================================
// Health
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn health_returns_ok(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/health").await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["status"], "healthy");
}

// ============================================================================
// Auth — Register
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn register_creates_user(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server
        .post("/api/auth/register")
        .json(&json!({
            "username": TEST_USER,
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    assert_eq!(body["data"]["username"], TEST_USER);
    assert_eq!(body["data"]["email"], TEST_EMAIL);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn register_duplicate_email_returns_conflict(pool: sqlx::PgPool) {
    let server = test_server(pool);
    // First registration succeeds.
    server
        .post("/api/auth/register")
        .json(&json!({
            "username": TEST_USER,
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;

    // Second registration with same email fails.
    let res = server
        .post("/api/auth/register")
        .json(&json!({
            "username": "other",
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn register_short_password_returns_validation_error(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server
        .post("/api/auth/register")
        .json(&json!({
            "username": TEST_USER,
            "email": TEST_EMAIL,
            "password": "short",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

// ============================================================================
// Auth — Login
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn login_returns_tokens(pool: sqlx::PgPool) {
    let server = test_server(pool);
    // Register first.
    server
        .post("/api/auth/register")
        .json(&json!({
            "username": TEST_USER,
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;

    let res = server
        .post("/api/auth/login")
        .json(&json!({
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["data"]["access_token"].is_string());
    assert!(body["data"]["refresh_token"].is_string());
    assert_eq!(body["data"]["token_type"], "Bearer");
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn login_wrong_password_returns_unauthorized(pool: sqlx::PgPool) {
    let server = test_server(pool);
    server
        .post("/api/auth/register")
        .json(&json!({
            "username": TEST_USER,
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        }))
        .await;

    let res = server
        .post("/api/auth/login")
        .json(&json!({
            "email": TEST_EMAIL,
            "password": "wrongpassword123",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn login_nonexistent_user_returns_unauthorized(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server
        .post("/api/auth/login")
        .json(&json!({
            "email": "nobody@example.com",
            "password": TEST_PASSWORD,
        }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Auth — Refresh
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn refresh_rotates_tokens(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (_, refresh_token) = register_and_login(&server).await;

    let res = server
        .post("/api/auth/refresh")
        .json(&json!({ "refresh_token": refresh_token }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body["data"]["access_token"].is_string());
    // New refresh token should be different.
    let new_refresh = body["data"]["refresh_token"].as_str().unwrap();
    assert_ne!(new_refresh, refresh_token);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn refresh_with_invalid_token_fails(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server
        .post("/api/auth/refresh")
        .json(&json!({ "refresh_token": "invalid-token" }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Auth — Me
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn me_returns_user_info(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;

    let res = server
        .get("/api/auth/me")
        .add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["username"], TEST_USER);
    assert_eq!(body["data"]["email"], TEST_EMAIL);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn me_without_token_returns_unauthorized(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/auth/me").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Banks — CRUD
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn banks_crud_lifecycle(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    // List — initially empty.
    let res = server
        .get("/api/banks")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // Create.
    let res = server
        .post("/api/banks")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Revolut" }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    let bank_id = body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["name"], "Revolut");

    // Get by ID.
    let res = server
        .get(&format!("/api/banks/{bank_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["name"], "Revolut");

    // Update.
    let res = server
        .put(&format!("/api/banks/{bank_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Revolut EU" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["name"], "Revolut EU");

    // List — should have one bank.
    let res = server
        .get("/api/banks")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    // Archive.
    let res = server
        .put(&format!("/api/banks/{bank_id}/archive"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status(axum::http::StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn banks_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/banks").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Accounts — CRUD
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn accounts_crud_lifecycle(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    // Create a bank first (accounts need a parent bank).
    let res = server
        .post("/api/banks")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "TestBank" }))
        .await;
    let bank_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // List — initially empty.
    let res = server
        .get("/api/accounts")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // Create account.
    let res = server
        .post("/api/accounts")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "bank_id": bank_id,
            "name": "Checking EUR",
            "currency": "EUR",
            "type": "checking",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    let account_id = body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["name"], "Checking EUR");
    assert_eq!(body["data"]["currency"], "EUR");

    // Get by ID.
    let res = server
        .get(&format!("/api/accounts/{account_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Checking EUR");

    // Update.
    let res = server
        .put(&format!("/api/accounts/{account_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Main EUR Account" }))
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Main EUR Account");

    // Filter by bank_id.
    let res = server
        .get(&format!("/api/accounts?bank_id={bank_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 1);

    // Archive.
    let res = server
        .put(&format!("/api/accounts/{account_id}/archive"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status(axum::http::StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn accounts_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/accounts").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Categories — CRUD + Bulk
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn categories_crud_lifecycle(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    // List — initially empty.
    let res = server
        .get("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // Create.
    let res = server
        .post("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "name": "Groceries",
            "category_type": "expense",
            "icon": "cart",
            "color": "#22c55e",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    let cat_id = body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["name"], "Groceries");

    // Get.
    let res = server
        .get(&format!("/api/categories/{cat_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Groceries");

    // Update.
    let res = server
        .put(&format!("/api/categories/{cat_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Food & Groceries" }))
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Food & Groceries");

    // Delete.
    let res = server
        .delete(&format!("/api/categories/{cat_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Verify deleted.
    let res = server
        .get(&format!("/api/categories/{cat_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn categories_bulk_create(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    let res = server
        .post("/api/categories/bulk")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "categories": [
                { "name": "Salary", "category_type": "income" },
                { "name": "Rent", "category_type": "expense" },
                { "name": "Utilities", "category_type": "expense" },
            ]
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn categories_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/categories").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Tags — CRUD + Bulk
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn tags_crud_lifecycle(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    // List — initially empty.
    let res = server
        .get("/api/tags")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // Create.
    let res = server
        .post("/api/tags")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "vacation", "color": "#3b82f6" }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    let tag_id = body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["name"], "vacation");

    // Get.
    let res = server
        .get(&format!("/api/tags/{tag_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();

    // Update.
    let res = server
        .put(&format!("/api/tags/{tag_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "holiday" }))
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "holiday");

    // Delete.
    let res = server
        .delete(&format!("/api/tags/{tag_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Verify deleted.
    let res = server
        .get(&format!("/api/tags/{tag_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn tags_bulk_create(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    let res = server
        .post("/api/tags/bulk")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "tags": [
                { "name": "recurring" },
                { "name": "one-time", "color": "#ef4444" },
            ]
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn tags_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/tags").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Settings — Get & Update
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn settings_get_returns_defaults(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    let res = server
        .get("/api/settings")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["locale"], "en-US");
    assert_eq!(body["data"]["timezone"], "UTC");
    assert_eq!(body["data"]["default_currency"], "USD");
    assert_eq!(body["data"]["theme"], "system");
    assert_eq!(body["data"]["ai_enabled"], false);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn settings_update_partial(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    // Update only theme and currency.
    let res = server
        .put("/api/settings")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "theme": "dark",
            "default_currency": "EUR",
        }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["theme"], "dark");
    assert_eq!(body["data"]["default_currency"], "EUR");
    // Unchanged fields remain at defaults.
    assert_eq!(body["data"]["locale"], "en-US");
    assert_eq!(body["data"]["timezone"], "UTC");
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn settings_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/settings").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// i18n — List Locales
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn i18n_list_locales_returns_available_locales(pool: sqlx::PgPool) {
    let server = test_server(pool);
    // This is a public endpoint — no auth needed.
    let res = server.get("/api/i18n/locales").await;
    res.assert_status_ok();
    let body: Value = res.json();
    let locales = body["data"].as_array().unwrap();
    assert!(!locales.is_empty());

    let en = &locales[0];
    assert_eq!(en["code"], "en-US");
    assert_eq!(en["name"], "English (US)");
    assert_eq!(en["completeness"], 100);
    assert_eq!(en["is_default"], true);
}

// ============================================================================
// Cross-cutting: Data Isolation Between Users
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn user_data_isolation(pool: sqlx::PgPool) {
    let server = test_server(pool);

    // Register and login user A.
    server
        .post("/api/auth/register")
        .json(&json!({
            "username": "userA",
            "email": "a@example.com",
            "password": TEST_PASSWORD,
        }))
        .await;
    let res = server
        .post("/api/auth/login")
        .json(&json!({
            "email": "a@example.com",
            "password": TEST_PASSWORD,
        }))
        .await;
    let token_a = res.json::<Value>()["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();
    let auth_a = format!("Bearer {token_a}");

    // Register and login user B.
    server
        .post("/api/auth/register")
        .json(&json!({
            "username": "userB",
            "email": "b@example.com",
            "password": TEST_PASSWORD,
        }))
        .await;
    let res = server
        .post("/api/auth/login")
        .json(&json!({
            "email": "b@example.com",
            "password": TEST_PASSWORD,
        }))
        .await;
    let token_b = res.json::<Value>()["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();
    let auth_b = format!("Bearer {token_b}");

    // User A creates a bank.
    let res = server
        .post("/api/banks")
        .add_header(axum::http::header::AUTHORIZATION, auth_a.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "A's Bank" }))
        .await;
    let bank_id_a = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // User B cannot see user A's banks.
    let res = server
        .get("/api/banks")
        .add_header(axum::http::header::AUTHORIZATION, auth_b.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // User B cannot access user A's bank by ID.
    let res = server
        .get(&format!("/api/banks/{bank_id_a}"))
        .add_header(axum::http::header::AUTHORIZATION, auth_b.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}
