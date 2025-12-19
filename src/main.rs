// src/main.rs
// DOCUMENTATION: Application entry point
// PURPOSE: Initialize config, database, and start HTTP server

mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod services;

use actix_web::{middleware::Logger, web, App, HttpServer};
use config::Config;
use dotenv::dotenv;
use std::io;
use std::sync::Arc;
use services::{PlacesCache, start_cleanup_task};

#[actix_web::main]
async fn main() -> io::Result<()> {
    // 1. Load environment variables
    dotenv().ok();

    // 2. Load configuration
    let config = Config::from_env();
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        // We continue but log error, or we could panic
    }

    // 3. Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        // Use configured log level or default
        let log_level = if !config.log_level.is_empty() {
            &config.log_level
        } else {
            "info,actix_web=info,sqlx=warn"
        };
        std::env::set_var("RUST_LOG", log_level);
    }
    env_logger::init();

    log::info!("Starting auphere-places microservice...");
    log::info!("Environment: {}", config.environment);
    log::info!(
        "Server Address: {}:{}",
        config.server_address,
        config.server_port
    );

    // 4. Initialize database connection pool
    let pool = match config::init_db_pool(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            log::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // 5. Initialize cache for Google Places API responses
    let cache = Arc::new(PlacesCache::new(3600)); // 1 hour TTL
    log::info!("Initialized Places API cache (TTL: 1 hour)");
    
    // Start background cleanup task (runs every 5 minutes)
    start_cleanup_task(cache.clone(), 300);
    log::info!("Started cache cleanup task (interval: 5 minutes)");

    // 6. Start HTTP server
    let server_addr = format!("{}:{}", config.server_address, config.server_port);
    let config_clone = config.clone();

    HttpServer::new(move || {
        App::new()
            // Application state (database pool, config, and cache)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config_clone.clone()))
            .app_data(web::Data::new(cache.clone()))
            // Middleware
            .wrap(Logger::default())
            .wrap(actix_web::middleware::Compress::default())
            // Routes
            .configure(handlers::health_config)
            .configure(handlers::places_config)
            .configure(handlers::admin_config)
    })
    .bind(&server_addr)?
    .run()
    .await
}
