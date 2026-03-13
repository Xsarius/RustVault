//! Integration tests for all P1 API endpoints.
//!
//! Uses `sqlx::test` for ephemeral databases and `axum-test` for
//! zero-overhead HTTP testing without a TCP listener.

mod helpers;

use axum_test::multipart::{MultipartForm, Part};
use helpers::{TEST_EMAIL, TEST_PASSWORD, TEST_USER, register_and_login, test_server};
use serde_json::{Value, json};
use uuid::Uuid;

async fn create_bank_and_account(server: &axum_test::TestServer, auth: &str) -> String {
    let suffix = Uuid::new_v4().to_string();
    let bank_res = server
        .post("/api/banks")
        .add_header(axum::http::header::AUTHORIZATION, auth)
        .json(&json!({ "name": format!("Import Test Bank {suffix}") }))
        .await;
    bank_res.assert_status(axum::http::StatusCode::CREATED);
    let bank_id = bank_res.json::<Value>()["data"]["id"]
        .as_str()
        .expect("bank id")
        .to_string();

    let account_res = server
        .post("/api/accounts")
        .add_header(axum::http::header::AUTHORIZATION, auth)
        .json(&json!({
            "bank_id": bank_id,
            "name": format!("Import Checking {suffix}"),
            "currency": "EUR",
            "type": "checking",
        }))
        .await;
    account_res.assert_status(axum::http::StatusCode::CREATED);

    account_res.json::<Value>()["data"]["id"]
        .as_str()
        .expect("account id")
        .to_string()
}

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
            format!("Bearer {token}")
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // Create.
    let res = server
        .post("/api/banks")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "Revolut" }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    let bank_id = body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["name"], "Revolut");

    // Get by ID.
    let res = server
        .get(&format!("/api/banks/{bank_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["name"], "Revolut");

    // Update.
    let res = server
        .put(&format!("/api/banks/{bank_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "Revolut EU" }))
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"]["name"], "Revolut EU");

    // List — should have one bank.
    let res = server
        .get("/api/banks")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    // Archive.
    let res = server
        .put(&format!("/api/banks/{bank_id}/archive"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "TestBank" }))
        .await;
    let bank_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // List — initially empty.
    let res = server
        .get("/api/accounts")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // Create account.
    let res = server
        .post("/api/accounts")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Checking EUR");

    // Update.
    let res = server
        .put(&format!("/api/accounts/{account_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "Main EUR Account" }))
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Main EUR Account");

    // Filter by bank_id.
    let res = server
        .get(&format!("/api/accounts?bank_id={bank_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 1);

    // Archive.
    let res = server
        .put(&format!("/api/accounts/{account_id}/archive"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // Create.
    let res = server
        .post("/api/categories")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Groceries");

    // Update.
    let res = server
        .put(&format!("/api/categories/{cat_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "Food & Groceries" }))
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "Food & Groceries");

    // Delete.
    let res = server
        .delete(&format!("/api/categories/{cat_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Verify deleted.
    let res = server
        .get(&format!("/api/categories/{cat_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // Create.
    let res = server
        .post("/api/tags")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "vacation", "color": "#3b82f6" }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    let tag_id = body["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["name"], "vacation");

    // Get.
    let res = server
        .get(&format!("/api/tags/{tag_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();

    // Update.
    let res = server
        .put(&format!("/api/tags/{tag_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "holiday" }))
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"]["name"], "holiday");

    // Delete.
    let res = server
        .delete(&format!("/api/tags/{tag_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Verify deleted.
    let res = server
        .get(&format!("/api/tags/{tag_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
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
// Imports — Upload, Execute, Rollback
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn imports_upload_returns_preview(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    let csv_bytes =
        b"date,amount,description\n2026-03-01,-12.34,Coffee\n2026-03-02,100.00,Salary\n";
    let form = MultipartForm::new()
        .add_text("account_id", account_id)
        .add_part(
            "file",
            Part::bytes(csv_bytes.as_slice())
                .file_name("statement.csv")
                .mime_type("text/csv"),
        );

    let res = server
        .post("/api/imports/upload")
        .add_header(axum::http::header::AUTHORIZATION, auth)
        .multipart(form)
        .await;

    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    assert_eq!(body["data"]["detected_format"], "csv");
    assert_eq!(body["data"]["total_rows"], 2);
    assert_eq!(body["data"]["preview"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn imports_upload_and_execute_then_rollback(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    let csv_bytes =
        b"date,amount,description\n2026-03-01,-12.34,Coffee\n2026-03-02,100.00,Salary\n";
    let form = MultipartForm::new()
        .add_text("account_id", account_id)
        .add_text("skip_duplicates", "true")
        .add_part(
            "file",
            Part::bytes(csv_bytes.as_slice())
                .file_name("statement.csv")
                .mime_type("text/csv"),
        );

    let res = server
        .post("/api/imports/upload-and-execute")
        .add_header(axum::http::header::AUTHORIZATION, &auth)
        .multipart(form)
        .await;
    res.assert_status_ok();
    let body: Value = res.json();

    assert_eq!(body["data"]["imported_count"], 2);
    let import_id = body["data"]["import"]["id"]
        .as_str()
        .expect("import id")
        .to_string();

    let list_res = server
        .get("/api/imports")
        .add_header(axum::http::header::AUTHORIZATION, &auth)
        .await;
    list_res.assert_status_ok();
    assert!(
        !list_res.json::<Value>()["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let get_res = server
        .get(&format!("/api/imports/{import_id}"))
        .add_header(axum::http::header::AUTHORIZATION, &auth)
        .await;
    get_res.assert_status_ok();
    assert_eq!(get_res.json::<Value>()["data"]["status"], "completed");

    let rollback_res = server
        .delete(&format!("/api/imports/{import_id}"))
        .add_header(axum::http::header::AUTHORIZATION, &auth)
        .await;
    rollback_res.assert_status(axum::http::StatusCode::NO_CONTENT);

    let get_after_rollback = server
        .get(&format!("/api/imports/{import_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth)
        .await;
    get_after_rollback.assert_status_ok();
    assert_eq!(
        get_after_rollback.json::<Value>()["data"]["status"],
        "rolled_back"
    );
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn imports_list_requires_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/imports").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Transactions — CRUD + Bulk
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transactions_crud_and_bulk(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    let create_res = server
        .post("/api/transactions")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "account_id": account_id,
            "category_id": null,
            "transaction_type": "expense",
            "amount": "-12.34",
            "date": { "year": 2026, "ordinal": 60 },
            "description": "Coffee shop",
            "payee": "Coffee Shop",
            "notes": "morning",
            "tag_ids": []
        }))
        .await;
    create_res.assert_status(axum::http::StatusCode::CREATED);
    let tx_id = create_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let list_res = server
        .get("/api/transactions?limit=20")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    list_res.assert_status_ok();
    assert!(
        !list_res.json::<Value>()["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let update_res = server
        .put(&format!("/api/transactions/{tx_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "description": "Coffee beans",
            "is_reviewed": true
        }))
        .await;
    update_res.assert_status_ok();
    assert_eq!(
        update_res.json::<Value>()["data"]["description"],
        "Coffee beans"
    );

    let bulk_res = server
        .patch("/api/transactions/bulk")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "transaction_ids": [tx_id],
            "is_reviewed": false,
            "add_tag_ids": []
        }))
        .await;
    bulk_res.assert_status_ok();
    assert_eq!(bulk_res.json::<Value>()["data"]["updated"], 1);

    let delete_res = server
        .delete(&format!("/api/transactions/{tx_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    delete_res.assert_status(axum::http::StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transactions_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/transactions").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Transfers — Create / Detect / Unlink
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transfers_create_detect_and_unlink(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    let account_a = create_bank_and_account(&server, &auth).await;
    let account_b = create_bank_and_account(&server, &auth).await;

    let create_transfer_res = server
        .post("/api/transfers")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "from_account_id": account_a,
            "to_account_id": account_b,
            "amount": "15.00",
            "date": { "year": 2026, "ordinal": 60 },
            "description": "Wallet top-up",
            "method": "internal"
        }))
        .await;
    create_transfer_res.assert_status(axum::http::StatusCode::CREATED);
    let transfer_id = create_transfer_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create a second pair of matching transactions so detect endpoint has candidates.
    let debit_res = server
        .post("/api/transactions")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "account_id": account_a,
            "transaction_type": "expense",
            "amount": "-20.00",
            "date": { "year": 2026, "ordinal": 61 },
            "description": "Manual transfer out",
            "tag_ids": []
        }))
        .await;
    debit_res.assert_status(axum::http::StatusCode::CREATED);

    let credit_res = server
        .post("/api/transactions")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "account_id": account_b,
            "transaction_type": "income",
            "amount": "20.00",
            "date": { "year": 2026, "ordinal": 61 },
            "description": "Manual transfer in",
            "tag_ids": []
        }))
        .await;
    credit_res.assert_status(axum::http::StatusCode::CREATED);

    let detect_res = server
        .post("/api/transfers/detect?date_tolerance_days=1&amount_tolerance=0")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    detect_res.assert_status_ok();
    assert!(detect_res.json::<Value>()["items"].is_array());

    let unlink_res = server
        .delete(&format!("/api/transfers/{transfer_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    unlink_res.assert_status(axum::http::StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transfers_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.post("/api/transfers/detect").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Rules — CRUD + Test + Suggest
// ============================================================================

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn rules_crud_test_and_suggest(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    let create_res = server
        .post("/api/rules")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "name": "Coffee Rule",
            "priority": 10,
            "conditions": [
                { "field": "description_contains", "value": "coffee", "logic": "and" }
            ],
            "actions": [
                { "type": "set_payee", "value": "Coffee Shop" }
            ]
        }))
        .await;
    create_res.assert_status(axum::http::StatusCode::CREATED);
    let rule_id = create_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let list_res = server
        .get("/api/rules")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    list_res.assert_status_ok();
    assert!(
        !list_res.json::<Value>()["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let get_res = server
        .get(&format!("/api/rules/{rule_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    get_res.assert_status_ok();
    assert_eq!(get_res.json::<Value>()["data"]["name"], "Coffee Rule");

    let update_res = server
        .put(&format!("/api/rules/{rule_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "Coffee Rule Updated", "is_enabled": true }))
        .await;
    update_res.assert_status_ok();
    assert_eq!(
        update_res.json::<Value>()["data"]["name"],
        "Coffee Rule Updated"
    );

    let test_res = server
        .post("/api/rules/test")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "conditions": [
                { "field": "description_contains", "value": "coffee", "logic": "and" },
                { "field": "account_id", "value": account_id, "logic": "and" }
            ],
            "description": "Coffee purchase",
            "payee": "Coffee Shop",
            "amount": "-4.20",
            "account_id": account_id
        }))
        .await;
    test_res.assert_status_ok();
    assert_eq!(test_res.json::<Value>()["data"]["matched"], true);

    let suggest_res = server
        .post("/api/rules/suggest")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({
            "description": "Spotify subscription",
            "payee": "Spotify",
            "amount": "-29.99"
        }))
        .await;
    suggest_res.assert_status_ok();
    assert!(suggest_res.json::<Value>()["data"]["conditions"].is_array());

    let delete_res = server
        .delete(&format!("/api/rules/{rule_id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    delete_res.assert_status(axum::http::StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn rules_require_auth(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let res = server.get("/api/rules").await;
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
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth_a.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .json(&json!({ "name": "A's Bank" }))
        .await;
    let bank_id_a = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // User B cannot see user A's banks.
    let res = server
        .get("/api/banks")
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth_b.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status_ok();
    assert_eq!(res.json::<Value>()["data"].as_array().unwrap().len(), 0);

    // User B cannot access user A's bank by ID.
    let res = server
        .get(&format!("/api/banks/{bank_id_a}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            auth_b.parse::<axum::http::HeaderValue>().unwrap(),
        )
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}
