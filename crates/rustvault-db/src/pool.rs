//! Database connection pool creation and migration runner.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use crate::DbResult;

/// Create a PostgreSQL connection pool and run embedded migrations.
pub async fn create_pool(database_url: &str, max_connections: u32) -> DbResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;

    info!("Connected to PostgreSQL");

    // Run embedded migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Migrations applied successfully");

    Ok(pool)
}
