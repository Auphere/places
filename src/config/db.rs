// src/config/db.rs
// DOCUMENTATION: Database connection pool initialization
// PURPOSE: Setup and manage PostgreSQL connection pool

use crate::config::Config;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Initialize PostgreSQL connection pool
/// DOCUMENTATION: Creates connection pool with optimal settings
/// Called once during application startup in main.rs
/// Returns pool that is used for all database operations
pub async fn init_db_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    log::info!("Initializing database pool: {}", config.database_url);

    let pool = PgPoolOptions::new()
        // Maximum concurrent connections
        .max_connections(config.db_max_connections)
        // Timeout waiting for connection from pool
        .acquire_timeout(Duration::from_secs(config.db_connection_timeout))
        // Connection idle timeout (5 minutes)
        .idle_timeout(Duration::from_secs(300))
        // Connection lifetime (30 minutes before recycle)
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.database_url)
        .await?;

    // Verify connection works
    sqlx::query("SELECT 1").execute(&pool).await?;

    log::info!("Database pool initialized successfully");
    Ok(pool)
}
