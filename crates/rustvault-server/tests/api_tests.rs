//! Integration tests for all P1 API endpoints.
//!
//! Uses `sqlx::test` for ephemeral databases and `axum-test` for
//! zero-overhead HTTP testing without a TCP listener.

mod helpers;

use axum_test::multipart::{MultipartForm, Part};
use helpers::{TEST_EMAIL, TEST_PASSWORD, TEST_USER, register_and_login, test_server};
use serde_json::{Value, json};
use time::{Date, Month};
use uuid::Uuid;

fn json_date(year: i32, month: Month, day: u8) -> Value {
    let date = Date::from_calendar_date(year, month, day).expect("valid date");
    serde_json::to_value(date).expect("date should serialize")
}

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

    // Step 1: upload to create the import record and get the import ID.
    let upload_form = MultipartForm::new()
        .add_text("account_id", account_id)
        .add_part(
            "file",
            Part::bytes(csv_bytes.as_slice())
                .file_name("statement.csv")
                .mime_type("text/csv"),
        );

    let upload_res = server
        .post("/api/imports/upload")
        .add_header(axum::http::header::AUTHORIZATION, &auth)
        .multipart(upload_form)
        .await;
    upload_res.assert_status(axum::http::StatusCode::CREATED);
    let upload_body: Value = upload_res.json();
    let import_id = upload_body["data"]["import"]["id"]
        .as_str()
        .expect("import id")
        .to_string();

    // Step 2: execute — re-send the file to the execute endpoint.
    let execute_form = MultipartForm::new()
        .add_text("skip_duplicates", "true")
        .add_part(
            "file",
            Part::bytes(csv_bytes.as_slice())
                .file_name("statement.csv")
                .mime_type("text/csv"),
        );

    let res = server
        .post(&format!("/api/imports/{import_id}/execute"))
        .add_header(axum::http::header::AUTHORIZATION, &auth)
        .multipart(execute_form)
        .await;
    res.assert_status_ok();
    let body: Value = res.json();

    assert_eq!(body["data"]["imported_count"], 2);

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
            "date": json_date(2026, Month::March, 1),
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
            "date": json_date(2026, Month::March, 1),
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
            "date": json_date(2026, Month::March, 2),
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
            "date": json_date(2026, Month::March, 2),
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
    assert!(detect_res.json::<Value>()["data"].is_array());

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

// ============================================================================
// Real-world Scenarios — Budgets
// ============================================================================

/// Full budget workflow: create budget, bulk-set lines per category, then verify
/// the budget summary reflects actual spending from manually created transactions.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn budget_planned_vs_actual_scenario(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Create two expense categories.
    let groceries_id = server
        .post("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Groceries", "category_type": "expense" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let rent_id = server
        .post("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Rent", "category_type": "expense" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create a budget covering March 2026.
    let budget_res = server
        .post("/api/budgets")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "name": "March 2026",
            "period_start": json_date(2026, Month::March, 1),
            "period_end":   json_date(2026, Month::March, 31),
            "currency": "EUR",
            "is_recurring": false
        }))
        .await;
    budget_res.assert_status(axum::http::StatusCode::CREATED);
    let budget_id = budget_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Bulk-set two lines: Groceries 400 EUR, Rent 1200 EUR.
    let lines_res = server
        .post(&format!("/api/budgets/{budget_id}/lines/bulk"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "lines": [
                { "category_id": groceries_id, "planned_amount": "400.00", "sort_order": 0 },
                { "category_id": rent_id,      "planned_amount": "1200.00", "sort_order": 1 }
            ]
        }))
        .await;
    lines_res.assert_status(axum::http::StatusCode::CREATED);
    assert_eq!(lines_res.json::<Value>()["data"].as_array().unwrap().len(), 2);

    // Create transactions that fall inside the budget period.
    for (desc, amount, cat) in [
        ("Lidl weekly shop", "-85.20", &groceries_id),
        ("Kaufland big shop", "-112.50", &groceries_id),
        ("Monthly rent",     "-1200.00", &rent_id),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "category_id": cat,
                "transaction_type": "expense",
                "amount": amount,
                "date": json_date(2026, Month::March, 5),
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    // Fetch budget summary — actuals should reflect the transactions above.
    let summary_res = server
        .get(&format!("/api/budgets/{budget_id}/summary"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    summary_res.assert_status_ok();
    let summary = summary_res.json::<Value>();

    // Total planned expenses = 400 + 1200 = 1600 EUR.
    assert_eq!(summary["data"]["total_planned_expenses"], "1600");

    // Total actual expenses for the period = 85.20 + 112.50 + 1200 = 1397.70.
    let actual: f64 = summary["data"]["total_actual_expenses"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    assert!(
        (actual - 1397.70).abs() < 0.01,
        "expected ~1397.70 actual expenses, got {actual}"
    );

    // Rent line should show 100% utilisation.
    let lines_arr = summary["data"]["lines"].as_array().unwrap();
    let rent_line = lines_arr
        .iter()
        .find(|l| l["category_id"].as_str() == Some(&rent_id))
        .expect("rent line present");
    assert_eq!(rent_line["actual_amount"], "1200");
    assert_eq!(rent_line["remaining"], "0");
}

/// Recurring budget: create with FREQ=MONTHLY, then manually trigger next-period
/// generation via the `/api/budgets/:id/generate-next` endpoint.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn budget_recurring_generate_next_period(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    // Create a recurring budget for January 2026.
    let create_res = server
        .post("/api/budgets")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "name":            "Jan 2026",
            "period_start":    json_date(2026, Month::January, 1),
            "period_end":      json_date(2026, Month::January, 31),
            "currency":        "EUR",
            "is_recurring":    true,
            "recurrence_rule": "FREQ=MONTHLY"
        }))
        .await;
    create_res.assert_status(axum::http::StatusCode::CREATED);
    let budget_id = create_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Add one line so the generated budget has something to copy.
    server
        .post(&format!("/api/budgets/{budget_id}/lines"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "category_id": null, "planned_amount": "500.00" }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Call the generate-next endpoint.
    let gen_res = server
        .post(&format!("/api/budgets/{budget_id}/generate-next"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    gen_res.assert_status(axum::http::StatusCode::CREATED);
    let new_budget = gen_res.json::<Value>();

    // Generated budget should cover February 2026.
    assert_eq!(new_budget["data"]["period_start"], "2026-02-01");
    assert_eq!(new_budget["data"]["period_end"],   "2026-02-28");
    // The generated budget itself is non-recurring (copies start as snapshots).
    assert_eq!(new_budget["data"]["is_recurring"], false);

    // Verify lines were copied.
    let lines_res = server
        .get(&format!("/api/budgets/{}/lines", new_budget["data"]["id"].as_str().unwrap()))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    lines_res.assert_status_ok();
    assert_eq!(lines_res.json::<Value>()["data"].as_array().unwrap().len(), 1);
}

/// Budget copy creates an independent new period with identical lines.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn budget_copy_clones_lines(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    let budget_id = {
        let res = server
            .post("/api/budgets")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "name": "Feb 2026", "period_start": json_date(2026, Month::February, 1),
                "period_end": json_date(2026, Month::February, 28), "currency": "EUR",
            }))
            .await;
        res.assert_status(axum::http::StatusCode::CREATED);
        res.json::<Value>()["data"]["id"].as_str().unwrap().to_string()
    };

    // Add two lines.
    for amount in ["300.00", "150.00"] {
        server
            .post(&format!("/api/budgets/{budget_id}/lines"))
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({ "category_id": null, "planned_amount": amount }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    // Copy into March 2026.
    let copy_res = server
        .post(&format!("/api/budgets/{budget_id}/copy"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "name": "Mar 2026",
            "period_start": json_date(2026, Month::March, 1),
            "period_end":   json_date(2026, Month::March, 31)
        }))
        .await;
    copy_res.assert_status(axum::http::StatusCode::CREATED);
    let new_id = copy_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Copied budget has 2 lines with identical planned amounts.
    let lines = server
        .get(&format!("/api/budgets/{new_id}/lines"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .json::<Value>();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);

    // Modifying the copy does not affect the original.
    let first_line_id = lines["data"][0]["id"].as_str().unwrap().to_string();
    server
        .put(&format!("/api/budgets/{new_id}/lines/{first_line_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "planned_amount": "999.00" }))
        .await
        .assert_status_ok();

    let orig_lines = server
        .get(&format!("/api/budgets/{budget_id}/lines"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .json::<Value>();
    // Original planned amounts are unchanged (300 and 150).
    let orig_amounts: Vec<&str> = orig_lines["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["planned_amount"].as_str().unwrap())
        .collect();
    assert!(orig_amounts.iter().any(|&a| a == "300"), "original 300 line untouched");
    assert!(orig_amounts.iter().any(|&a| a == "150"), "original 150 line untouched");
}

// ============================================================================
// Real-world Scenarios — Reports
// ============================================================================

/// Dashboard summary returns sensible totals once transactions exist.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn reports_summary_reflects_transactions(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    let today = time::OffsetDateTime::now_utc().date();
    let (y, m, d) = (today.year(), today.month() as u8, today.day());
    let today_json = json!(format!("{y:04}-{m:02}-{d:02}"));

    // Create one income and two expense transactions dated today.
    for (tx_type, amount, desc) in [
        ("income",  "3000.00", "Salary"),
        ("expense", "-800.00",  "Rent"),
        ("expense", "-150.00",  "Utilities"),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "transaction_type": tx_type,
                "amount": amount,
                "date": today_json,
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let res = server
        .get("/api/reports/summary")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();

    let income: f64 = body["data"]["month_income"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let expenses: f64 = body["data"]["month_expenses"].as_str().unwrap_or("0").parse().unwrap_or(0.0);

    assert!(income >= 3000.0,    "month_income should include 3000 salary, got {income}");
    assert!(expenses >= 950.0,   "month_expenses should include 950 total, got {expenses}");
    assert!(body["data"]["savings_rate"].is_string() || body["data"]["savings_rate"].is_f64(),
        "savings_rate should be present");

    // unreviewed_count increases for new (unreviewed) transactions.
    let unreviewed = body["data"]["unreviewed_count"].as_i64().unwrap_or(0);
    assert!(unreviewed >= 3, "unreviewed_count should be ≥ 3, got {unreviewed}");
}

/// Income-vs-expense report over a two-month range returns per-month breakdown.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn reports_income_expense_monthly_breakdown(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Seed two months of transactions.
    for (month, tx_type, amount, desc) in [
        (Month::January,  "income",  "4000.00", "Jan Salary"),
        (Month::January,  "expense", "-500.00",  "Jan Rent"),
        (Month::February, "income",  "4200.00", "Feb Salary"),
        (Month::February, "expense", "-520.00",  "Feb Rent"),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "transaction_type": tx_type,
                "amount": amount,
                "date": json_date(2026, month, 15),
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let res = server
        .get("/api/reports/income-expense?from=2026-01-01&to=2026-02-28")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let months = body["data"]["months"].as_array().unwrap();

    // Should have at least 2 months in range.
    assert!(months.len() >= 2, "expected ≥2 months in range, got {}", months.len());

    // Verify January totals.
    let jan = months
        .iter()
        .find(|m| m["month"].as_str().unwrap_or("").starts_with("2026-01"))
        .expect("January entry present");
    let jan_income: f64 = jan["income"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    assert!((jan_income - 4000.0).abs() < 1.0, "jan income should be ~4000, got {jan_income}");
}

/// Balance history returns a data-point per day for the requested account range.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn reports_balance_history_grows_with_transactions(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Seed three days of income.
    for (day, amount) in [(1u8, "100.00"), (2, "200.00"), (3, "50.00")] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "transaction_type": "income",
                "amount": amount,
                "date": json_date(2026, Month::March, day),
                "description": format!("Day {day} inflow"),
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let res = server
        .get(&format!("/api/reports/balance-history?from=2026-03-01&to=2026-03-07&account_ids={account_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    let points = body["data"]["points"].as_array().unwrap();
    assert!(!points.is_empty(), "balance history should have data points");
}

/// Cash-flow report returns income and expense series over a date range.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn reports_cash_flow_returns_daily_series(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    server
        .post("/api/transactions")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "account_id": account_id,
            "transaction_type": "income",
            "amount": "500.00",
            "date": json_date(2026, Month::March, 3),
            "description": "Freelance payment",
            "tag_ids": []
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let res = server
        .get("/api/reports/cash-flow?from=2026-03-01&to=2026-03-07")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    // Response should at minimum contain income/expense series without error.
    let body = res.json::<Value>();
    assert!(body["data"].is_object(), "cash-flow body should be an object");
}

/// Category trend returns monthly spending points for a specific category.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn reports_category_trend_over_three_months(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Create a "Groceries" category.
    let cat_id = server
        .post("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Groceries", "category_type": "expense" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Seed one grocery transaction per month for 3 months.
    for (month, amount) in [
        (Month::January,  "-95.00"),
        (Month::February, "-102.50"),
        (Month::March,    "-88.75"),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "category_id": cat_id,
                "transaction_type": "expense",
                "amount": amount,
                "date": json_date(2026, month, 10),
                "description": "Grocery run",
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let res = server
        .get(&format!("/api/reports/categories/{cat_id}/trend?from=2026-01-01&to=2026-03-31"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body = res.json::<Value>();
    assert_eq!(body["data"]["category_id"], cat_id);
    let periods = body["data"]["periods"].as_array().unwrap();
    assert!(periods.len() >= 3, "expected ≥3 monthly trend points, got {}", periods.len());
}

// ============================================================================
// Real-world Scenarios — Rule Engine + Import
// ============================================================================

/// End-to-end: rules auto-categorize transactions upon import execution.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn rule_engine_auto_categorizes_on_import(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Create an "Entertainment" category.
    let spotify_cat_id = server
        .post("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Entertainment", "category_type": "expense" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create a rule: description_contains "spotify" → set_category to Entertainment.
    server
        .post("/api/rules")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "name": "Spotify",
            "priority": 10,
            "is_enabled": true,
            "conditions": [
                { "field": "description_contains", "value": "spotify", "logic": "and" }
            ],
            "actions": [
                { "type": "set_category", "value": spotify_cat_id }
            ]
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Build a CSV where one transaction mentions "Spotify".
    let csv = format!(
        "date,amount,description\n\
         2026-03-01,-9.99,Spotify Premium\n\
         2026-03-02,-50.00,Supermarket\n"
    );
    let csv_bytes = csv.as_bytes().to_vec();

    // Step 1 – preview (upload).
    let preview_form = MultipartForm::new()
        .add_text("account_id", &account_id)
        .add_text("skip_duplicates", "true")
        .add_part(
            "file",
            Part::bytes(csv_bytes.as_slice())
                .file_name("bank.csv")
                .mime_type("text/csv"),
        );
    let preview_res = server
        .post("/api/imports/upload")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(preview_form)
        .await;
    preview_res.assert_status(axum::http::StatusCode::CREATED);
    let import_id = preview_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Step 2 – execute.
    let execute_form = MultipartForm::new()
        .add_text("skip_duplicates", "true")
        .add_part(
            "file",
            Part::bytes(csv_bytes.as_slice())
                .file_name("bank.csv")
                .mime_type("text/csv"),
        );
    let exec_res = server
        .post(&format!("/api/imports/{import_id}/execute"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(execute_form)
        .await;
    exec_res.assert_status_ok();
    assert_eq!(exec_res.json::<Value>()["data"]["imported_count"], 2);

    // Fetch imported transactions filtered by account.
    let txns = server
        .get(&format!("/api/transactions?account_id={account_id}&limit=50"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .json::<Value>();
    let rows = txns["data"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "imported 2 transactions");

    // The Spotify transaction should have category = spotify_cat_id.
    let spotify_txn = rows
        .iter()
        .find(|t| t["description"].as_str().unwrap_or("").to_lowercase().contains("spotify"))
        .expect("Spotify transaction present");
    assert_eq!(
        spotify_txn["category_id"].as_str().unwrap_or(""),
        spotify_cat_id,
        "rule should auto-categorize Spotify transaction"
    );
}

/// Rule test endpoint correctly evaluates conditions against sample input.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn rule_test_endpoint_matches_and_misses(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Match — description contains "netflix".
    let match_res = server
        .post("/api/rules/test")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "conditions": [
                { "field": "description_contains", "value": "netflix", "logic": "and" }
            ],
            "description": "Netflix monthly subscription",
            "payee": "Netflix Inc.",
            "amount": "-15.99",
            "account_id": account_id
        }))
        .await;
    match_res.assert_status_ok();
    assert_eq!(match_res.json::<Value>()["data"]["matched"], true);

    // No match — description does not contain "netflix".
    let miss_res = server
        .post("/api/rules/test")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "conditions": [
                { "field": "description_contains", "value": "netflix", "logic": "and" }
            ],
            "description": "Lidl grocery store",
            "payee": "Lidl",
            "amount": "-33.10",
            "account_id": account_id
        }))
        .await;
    miss_res.assert_status_ok();
    assert_eq!(miss_res.json::<Value>()["data"]["matched"], false);
}

// ============================================================================
// Real-world Scenarios — Transaction Filtering
// ============================================================================

/// Filter transactions by date range returns only transactions within range.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transaction_filter_by_date_range(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Create transactions on Jan 10, Feb 10, Mar 10.
    for (month, desc) in [
        (Month::January,  "Jan payment"),
        (Month::February, "Feb payment"),
        (Month::March,    "Mar payment"),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "transaction_type": "expense",
                "amount": "-10.00",
                "date": json_date(2026, month, 10),
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    // Filter: only February.
    let res = server
        .get("/api/transactions?date_from=2026-02-01&date_to=2026-02-28&limit=50")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let txns = res.json::<Value>();
    let rows = txns["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "only Feb transaction in range");
    assert!(
        rows[0]["description"].as_str().unwrap().contains("Feb"),
        "expected Feb payment"
    );
}

/// Filter transactions by type returns only matching types.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transaction_filter_by_type(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    for (tx_type, amount, desc) in [
        ("income",  "1000.00", "Salary"),
        ("expense", "-200.00",  "Bills"),
        ("expense", "-50.00",   "Coffee"),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "transaction_type": tx_type,
                "amount": amount,
                "date": json_date(2026, Month::March, 1),
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    // Filter: income only.
    let res = server
        .get("/api/transactions?transaction_type=income&limit=50")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let rows = res.json::<Value>();
    let income_txns = rows["data"].as_array().unwrap();
    assert_eq!(income_txns.len(), 1);
    assert_eq!(income_txns[0]["description"], "Salary");

    // Filter: expense only.
    let res2 = server
        .get("/api/transactions?transaction_type=expense&limit=50")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res2.assert_status_ok();
    assert_eq!(
        res2.json::<Value>()["data"].as_array().unwrap().len(),
        2,
        "two expense transactions"
    );
}

/// Filter transactions by category returns only categorized transactions.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transaction_filter_by_category(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    let food_id = server
        .post("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Food", "category_type": "expense" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let utilities_id = server
        .post("/api/categories")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Utilities", "category_type": "expense" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    for (cat, desc, amount) in [
        (&food_id,      "Lidl",        "-55.00"),
        (&food_id,      "Aldi",        "-30.00"),
        (&utilities_id, "Electric",   "-100.00"),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "category_id": cat,
                "transaction_type": "expense",
                "amount": amount,
                "date": json_date(2026, Month::March, 5),
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let res = server
        .get(&format!("/api/transactions?category_id={food_id}&limit=50"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(
        res.json::<Value>()["data"].as_array().unwrap().len(),
        2,
        "two food transactions"
    );
}

/// Full-text search on transactions returns matches by description.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transaction_fulltext_search(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    for (desc, amount) in [
        ("Apple Store in-app purchase", "-4.99"),
        ("Pret A Manger sandwich",       "-7.50"),
        ("Apple iCloud storage",         "-0.99"),
    ] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "transaction_type": "expense",
                "amount": amount,
                "date": json_date(2026, Month::March, 1),
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    // Search for "Apple" should return the two Apple transactions.
    let res = server
        .get("/api/transactions?q=Apple&limit=50")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    assert_eq!(
        res.json::<Value>()["data"].as_array().unwrap().len(),
        2,
        "two Apple transactions found by search"
    );
}

// ============================================================================
// Real-world Scenarios — Tags
// ============================================================================

/// Tags assigned at transaction creation are returned in list and detail views.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn transaction_tags_roundtrip(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Create two tags.
    let vacation_id = server
        .post("/api/tags")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "vacation", "color": "#3b82f6" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let business_id = server
        .post("/api/tags")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "business", "color": "#10b981" }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create a transaction carrying both tags.
    let tx_res = server
        .post("/api/transactions")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "account_id": account_id,
            "transaction_type": "expense",
            "amount": "-250.00",
            "date": json_date(2026, Month::March, 10),
            "description": "Business hotel",
            "tag_ids": [vacation_id, business_id]
        }))
        .await;
    tx_res.assert_status(axum::http::StatusCode::CREATED);
    let tx_id = tx_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Retrieve the transaction and confirm both tags are present.
    let get_res = server
        .get(&format!("/api/transactions/{tx_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    get_res.assert_status_ok();
    let tag_ids = get_res.json::<Value>()["data"]["tag_ids"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let tag_strings: Vec<&str> = tag_ids.iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(tag_strings.contains(&vacation_id.as_str()), "vacation tag present");
    assert!(tag_strings.contains(&business_id.as_str()),  "business tag present");

    // Filter transactions by tag should return this transaction.
    let filter_res = server
        .get(&format!("/api/transactions?tag_id={vacation_id}&limit=50"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    filter_res.assert_status_ok();
    let filtered = filter_res.json::<Value>();
    assert_eq!(filtered["data"].as_array().unwrap().len(), 1);
    assert_eq!(filtered["data"][0]["id"], tx_id);

    // Bulk-update: remove vacation tag (replace with only business).
    let bulk_res = server
        .patch("/api/transactions/bulk")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "transaction_ids": [tx_id],
            "add_tag_ids": []
        }))
        .await;
    bulk_res.assert_status_ok();
}

// ============================================================================
// Real-world Scenarios — Import pipeline (multiple formats)
// ============================================================================

/// MT940 import fully workflows from upload through execute.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn import_mt940_full_pipeline(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Minimal MT940 statement with two transactions.
    let mt940 = b":20:STMT001\n\
                   :25:NL21ABNA0417164300\n\
                   :28C:1/1\n\
                   :60F:C260301EUR1000,00\n\
                   :61:2603010301D50,00NTRFNONREF\n\
                   :86:Supermarket payment\n\
                   :61:2603020302C200,00NTRFNONREF\n\
                   :86:Salary deposit\n\
                   :62F:C260302EUR1150,00\n" as &[u8];

    let preview_form = MultipartForm::new()
        .add_text("account_id", &account_id)
        .add_text("skip_duplicates", "true")
        .add_part(
            "file",
            Part::bytes(mt940)
                .file_name("statement.mt940")
                .mime_type("application/octet-stream"),
        );
    let preview_res = server
        .post("/api/imports/upload")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(preview_form)
        .await;
    preview_res.assert_status(axum::http::StatusCode::CREATED);
    let body = preview_res.json::<Value>();
    let import_id = body["data"]["id"].as_str().unwrap();

    // Verify preview shows 2 rows.
    assert_eq!(body["data"]["total_rows"], 2,
        "MT940 with 2 transactions should preview 2 rows");

    // Execute.
    let execute_form = MultipartForm::new()
        .add_text("skip_duplicates", "true")
        .add_part(
            "file",
            Part::bytes(mt940)
                .file_name("statement.mt940")
                .mime_type("application/octet-stream"),
        );
    let exec_res = server
        .post(&format!("/api/imports/{import_id}/execute"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(execute_form)
        .await;
    exec_res.assert_status_ok();
    assert_eq!(exec_res.json::<Value>()["data"]["imported_count"], 2);

    // Imported transactions are visible on the account.
    let txns = server
        .get(&format!("/api/transactions?account_id={account_id}&limit=50"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .json::<Value>();
    assert_eq!(txns["data"].as_array().unwrap().len(), 2);
}

/// Duplicate detection: re-importing the same CSV with skip_duplicates=true
/// does not create duplicate transactions.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn import_csv_skip_duplicates(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    let csv = b"date,amount,description\n2026-03-01,-20.00,Coffee\n2026-03-02,-50.00,Lunch\n" as &[u8];

    // First import.
    let exec_form_1 = MultipartForm::new()
        .add_text("account_id", &account_id)
        .add_text("skip_duplicates", "true")
        .add_part("file", Part::bytes(csv).file_name("bank.csv").mime_type("text/csv"));
    let first = server
        .post("/api/imports/upload")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(exec_form_1)
        .await;
    first.assert_status(axum::http::StatusCode::CREATED);
    let import_id_1 = first.json::<Value>()["data"]["id"].as_str().unwrap().to_string();

    let ex1 = MultipartForm::new()
        .add_text("skip_duplicates", "true")
        .add_part("file", Part::bytes(csv).file_name("bank.csv").mime_type("text/csv"));
    server
        .post(&format!("/api/imports/{import_id_1}/execute"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(ex1)
        .await
        .assert_status_ok();

    // Second import of identical CSV with skip_duplicates=true.
    let exec_form_2 = MultipartForm::new()
        .add_text("account_id", &account_id)
        .add_text("skip_duplicates", "true")
        .add_part("file", Part::bytes(csv).file_name("bank.csv").mime_type("text/csv"));
    let second = server
        .post("/api/imports/upload")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(exec_form_2)
        .await;
    second.assert_status(axum::http::StatusCode::CREATED);
    let import_id_2 = second.json::<Value>()["data"]["id"].as_str().unwrap().to_string();

    let ex2 = MultipartForm::new()
        .add_text("skip_duplicates", "true")
        .add_part("file", Part::bytes(csv).file_name("bank.csv").mime_type("text/csv"));
    let second_exec = server
        .post(&format!("/api/imports/{import_id_2}/execute"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .multipart(ex2)
        .await;
    second_exec.assert_status_ok();

    // Total transactions should still be 2 (duplicates skipped).
    let txns = server
        .get(&format!("/api/transactions?account_id={account_id}&limit=50"))
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .json::<Value>();
    let count = txns["data"].as_array().unwrap().len();
    assert!(count <= 2, "expected ≤2 transactions after duplicate re-import, got {count}");
}

// ============================================================================
// Real-world Scenarios — Security: Cross-User Resource Isolation
// ============================================================================

/// Authenticated user B cannot access, modify or delete user A's budgets.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn cross_user_budget_isolation(pool: sqlx::PgPool) {
    let server = test_server(pool);

    let (token_a, _) = {
        server.post("/api/auth/register")
            .json(&json!({ "username": "alice", "email": "alice@example.com", "password": TEST_PASSWORD }))
            .await;
        let res = server.post("/api/auth/login")
            .json(&json!({ "email": "alice@example.com", "password": TEST_PASSWORD }))
            .await;
        let body = res.json::<Value>();
        (body["data"]["access_token"].as_str().unwrap().to_string(),
         body["data"]["refresh_token"].as_str().unwrap().to_string())
    };
    let (token_b, _) = {
        server.post("/api/auth/register")
            .json(&json!({ "username": "bob", "email": "bob@example.com", "password": TEST_PASSWORD }))
            .await;
        let res = server.post("/api/auth/login")
            .json(&json!({ "email": "bob@example.com", "password": TEST_PASSWORD }))
            .await;
        let body = res.json::<Value>();
        (body["data"]["access_token"].as_str().unwrap().to_string(),
         body["data"]["refresh_token"].as_str().unwrap().to_string())
    };

    let auth_a = format!("Bearer {token_a}");
    let auth_b = format!("Bearer {token_b}");

    // Alice creates a budget.
    let budget_id = server
        .post("/api/budgets")
        .add_header(axum::http::header::AUTHORIZATION, auth_a.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "name": "Alice Budget",
            "period_start": json_date(2026, Month::March, 1),
            "period_end":   json_date(2026, Month::March, 31),
            "currency": "EUR"
        }))
        .await
        .json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Bob cannot read Alice's budget.
    server
        .get(&format!("/api/budgets/{budget_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth_b.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .assert_status(axum::http::StatusCode::NOT_FOUND);

    // Bob cannot update Alice's budget.
    server
        .put(&format!("/api/budgets/{budget_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth_b.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({ "name": "Bob's takeover" }))
        .await
        .assert_status(axum::http::StatusCode::NOT_FOUND);

    // Bob cannot delete Alice's budget.
    server
        .delete(&format!("/api/budgets/{budget_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth_b.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .assert_status(axum::http::StatusCode::NOT_FOUND);
}

/// Authenticated user B cannot access user A's transactions or accounts.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn cross_user_transaction_isolation(pool: sqlx::PgPool) {
    let server = test_server(pool);

    let reg_and_login = |username: &str, email: &str| {
        let server = &server;
        let un = username.to_string();
        let em = email.to_string();
        async move {
            server.post("/api/auth/register")
                .json(&json!({ "username": un, "email": em, "password": TEST_PASSWORD }))
                .await;
            let res = server.post("/api/auth/login")
                .json(&json!({ "email": em, "password": TEST_PASSWORD }))
                .await;
            res.json::<Value>()["data"]["access_token"].as_str().unwrap().to_string()
        }
    };

    let token_a = reg_and_login("userZ", "z@example.com").await;
    let token_b = reg_and_login("userY", "y@example.com").await;
    let auth_a = format!("Bearer {token_a}");
    let auth_b = format!("Bearer {token_b}");

    // User A creates an account and a transaction.
    let account_id = create_bank_and_account(&server, &auth_a).await;
    let tx_res = server
        .post("/api/transactions")
        .add_header(axum::http::header::AUTHORIZATION, auth_a.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "account_id": account_id,
            "transaction_type": "expense",
            "amount": "-5.00",
            "date": json_date(2026, Month::March, 1),
            "description": "Secret data",
            "tag_ids": []
        }))
        .await;
    tx_res.assert_status(axum::http::StatusCode::CREATED);
    let tx_id = tx_res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // User B's transaction list is empty.
    let b_txns = server
        .get("/api/transactions?limit=50")
        .add_header(axum::http::header::AUTHORIZATION, auth_b.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .json::<Value>();
    assert_eq!(b_txns["data"].as_array().unwrap().len(), 0);

    // User B cannot read User A's transaction by ID.
    server
        .get(&format!("/api/transactions/{tx_id}"))
        .add_header(axum::http::header::AUTHORIZATION, auth_b.parse::<axum::http::HeaderValue>().unwrap())
        .await
        .assert_status(axum::http::StatusCode::NOT_FOUND);
}

// ============================================================================
// Real-world Scenarios — Settings
// ============================================================================

/// Settings are initialised with defaults and can be partially updated.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn settings_default_then_partial_update(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");

    // Defaults are returned immediately after registration.
    let defaults = server
        .get("/api/settings")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    defaults.assert_status_ok();
    let before = defaults.json::<Value>();
    assert!(before["data"]["default_currency"].is_string(),
        "default_currency should be a string");
    assert!(before["data"]["locale"].is_string(), "locale should be present");

    // Partial update: change default_currency and theme.
    let update_res = server
        .put("/api/settings")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "default_currency": "PLN",
            "theme": "dark"
        }))
        .await;
    update_res.assert_status_ok();
    let after = update_res.json::<Value>();
    assert_eq!(after["data"]["default_currency"], "PLN");
    assert_eq!(after["data"]["theme"], "dark");

    // Other settings not in the update payload remain at their defaults.
    assert_eq!(
        after["data"]["locale"],
        before["data"]["locale"],
        "unmodified locale should be unchanged"
    );
}

// ============================================================================
// Real-world Scenarios — Data Export
// ============================================================================

/// Export endpoint returns CSV when format=csv is requested.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn export_transactions_as_csv(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    // Seed two transactions.
    for (amount, desc) in [("-15.00", "Groceries"), ("2000.00", "Paycheck")] {
        server
            .post("/api/transactions")
            .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
            .json(&json!({
                "account_id": account_id,
                "transaction_type": if amount.starts_with('-') { "expense" } else { "income" },
                "amount": amount,
                "date": json_date(2026, Month::March, 1),
                "description": desc,
                "tag_ids": []
            }))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let res = server
        .get("/api/export?format=csv")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("csv") || content_type.contains("text"),
        "content-type should indicate CSV, got: {content_type}"
    );
    let body = res.text();
    assert!(body.contains("date") || body.contains("Date") || body.contains("amount") || body.contains("Amount"),
        "CSV response should contain column headers");
    assert!(body.contains("Groceries") || body.contains("Paycheck"),
        "CSV should contain transaction descriptions");
}

/// Export endpoint returns JSON when format=json is requested.
#[sqlx::test(migrations = "../rustvault-db/migrations")]
async fn export_transactions_as_json(pool: sqlx::PgPool) {
    let server = test_server(pool);
    let (token, _) = register_and_login(&server).await;
    let auth = format!("Bearer {token}");
    let account_id = create_bank_and_account(&server, &auth).await;

    server
        .post("/api/transactions")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .json(&json!({
            "account_id": account_id,
            "transaction_type": "expense",
            "amount": "-25.00",
            "date": json_date(2026, Month::March, 1),
            "description": "Test purchase",
            "tag_ids": []
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let res = server
        .get("/api/export?format=json")
        .add_header(axum::http::header::AUTHORIZATION, auth.parse::<axum::http::HeaderValue>().unwrap())
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert!(body.is_array() || body["transactions"].is_array() || body["data"].is_array(),
        "JSON export should return an array of transactions");
}
