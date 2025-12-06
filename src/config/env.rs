// src/config/env.rs
// DOCUMENTATION: Environment variable management
// PURPOSE: Load and validate configuration from .env files

use dotenv::dotenv;
use std::env;

/// Application configuration loaded from environment variables
/// DOCUMENTATION: Centralizes all configuration in one struct
/// Load with Config::from_env() at application startup
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL connection string
    /// Format: postgresql://user:password@host:port/database
    pub database_url: String,

    /// Server bind address (e.g., "127.0.0.1")
    pub server_address: String,

    /// Server listen port (default 8002)
    pub server_port: u16,

    /// Environment: development, staging, production
    pub environment: String,

    /// Log level: debug, info, warn, error
    pub log_level: String,

    /// Google Places API Key
    pub google_places_api_key: String,

    /// Admin authentication token (for sensitive endpoints)
    pub admin_token: String,

    /// Maximum connections in database pool
    pub db_max_connections: u32,

    /// Connection timeout in seconds
    pub db_connection_timeout: u64,
}

impl Config {
    /// Load configuration from environment variables
    /// DOCUMENTATION: Reads from .env.local or process environment
    /// Called once at application startup
    pub fn from_env() -> Self {
        // Load .env.local file if it exists
        dotenv().ok();

        Config {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://auphere:auphere@localhost:5432/places".to_string()
            }),

            server_address: env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),

            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8002".to_string())
                .parse()
                .unwrap_or(8002),

            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),

            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),

            google_places_api_key: env::var("GOOGLE_PLACES_API_KEY")
                .unwrap_or_else(|_| String::new()),

            admin_token: env::var("ADMIN_TOKEN").unwrap_or_else(|_| "admin-token-dev".to_string()),

            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),

            db_connection_timeout: env::var("DB_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }

    /// Validate critical configuration
    /// DOCUMENTATION: Ensures application can start safely
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL is required".to_string());
        }

        if self.google_places_api_key.is_empty() {
            log::warn!("GOOGLE_PLACES_API_KEY not configured - sync will not work");
        }

        Ok(())
    }
}
