// src/handlers/admin.rs
// DOCUMENTATION: Admin handlers for sync operations
// PURPOSE: Expose sync functionality via REST endpoints

use crate::config::Config;
use crate::errors::PlacesError;
use crate::services::{GooglePlacesClient, SyncService};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Request body for sync endpoint
#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    /// Optional place type filter (e.g., "restaurant", "bar")
    pub place_type: Option<String>,
    /// Optional grid cell size in kilometers
    pub cell_size_km: Option<f64>,
    /// Optional search radius in meters
    pub radius_m: Option<u32>,
}

/// Response for sync status endpoint
#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    /// Message describing sync status
    pub message: String,
    /// Total places in database
    pub total_places: i64,
    /// Places added in last 24 hours
    pub recent_additions: i64,
    /// Number of active places
    pub active_places: i64,
}

/// POST /admin/sync/{city}
/// Trigger synchronization for a city
///
/// DOCUMENTATION: Initiates Google Places sync for specified city
/// Requires admin authentication via X-Admin-Token header
pub async fn sync_city(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<SyncRequest>,
) -> Result<impl Responder, PlacesError> {
    // Authenticate admin request
    verify_admin_token(&req, &config)?;

    let city = path.into_inner();

    log::info!("Admin sync requested for city: {}", city);

    // Create Google Places client
    if config.google_places_api_key.is_empty() {
        return Err(PlacesError::InvalidInput(
            "Google Places API key not configured".to_string(),
        ));
    }

    let google_client = GooglePlacesClient::new(config.google_places_api_key.clone());

    // Execute sync
    let stats = SyncService::sync_city(
        pool.get_ref(),
        &google_client,
        &city,
        body.place_type.as_deref(),
        body.cell_size_km,
        body.radius_m,
    )
    .await?;

    log::info!(
        "Sync completed for {}: {} created, {} skipped, {} failed",
        city,
        stats.places_created,
        stats.places_skipped,
        stats.places_failed
    );

    Ok(HttpResponse::Ok().json(stats))
}

/// POST /admin/sync/batch
/// Trigger synchronization for multiple cities
///
/// DOCUMENTATION: Batch sync operation for multiple cities
#[derive(Debug, Deserialize)]
pub struct BatchSyncRequest {
    /// List of city names to sync
    pub cities: Vec<String>,
    /// Optional place type filter
    pub place_type: Option<String>,
}

pub async fn sync_cities_batch(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<BatchSyncRequest>,
) -> Result<impl Responder, PlacesError> {
    // Authenticate admin request
    verify_admin_token(&req, &config)?;

    log::info!(
        "Admin batch sync requested for {} cities",
        body.cities.len()
    );

    if config.google_places_api_key.is_empty() {
        return Err(PlacesError::InvalidInput(
            "Google Places API key not configured".to_string(),
        ));
    }

    let google_client = GooglePlacesClient::new(config.google_places_api_key.clone());

    // Execute batch sync
    let stats_list = SyncService::sync_cities(
        pool.get_ref(),
        &google_client,
        &body.cities,
        body.place_type.as_deref(),
    )
    .await;

    // Aggregate statistics
    let aggregated = SyncService::aggregate_stats(&stats_list);

    log::info!(
        "Batch sync completed: {} total created, {} total skipped, {} total failed",
        aggregated.places_created,
        aggregated.places_skipped,
        aggregated.places_failed
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "summary": aggregated,
        "details": stats_list,
    })))
}

/// GET /admin/sync/status
/// Get sync status and database statistics
///
/// DOCUMENTATION: Returns current system status and place counts
pub async fn sync_status(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
) -> Result<impl Responder, PlacesError> {
    // Authenticate admin request
    verify_admin_token(&req, &config)?;

    // Query database statistics
    let total_places: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM places")
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

    let active_places: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM places WHERE is_active = true")
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

    let recent_additions: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM places WHERE created_at > NOW() - INTERVAL '24 hours'",
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

    let response = SyncStatusResponse {
        message: "Sync service operational".to_string(),
        total_places: total_places.0,
        active_places: active_places.0,
        recent_additions: recent_additions.0,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /admin/stats
/// Get detailed database statistics
///
/// DOCUMENTATION: Returns comprehensive statistics about places
pub async fn database_stats(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
) -> Result<impl Responder, PlacesError> {
    // Authenticate admin request
    verify_admin_token(&req, &config)?;

    // Query statistics by type
    #[derive(Debug, Serialize, sqlx::FromRow)]
    struct TypeCount {
        #[sqlx(rename = "type")]
        type_field: Option<String>,
        count: Option<i64>,
    }

    let type_counts: Vec<TypeCount> = sqlx::query_as(
        "SELECT type, COUNT(*) as count FROM places WHERE is_active = true GROUP BY type ORDER BY count DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

    // Query statistics by city
    #[derive(Debug, Serialize, sqlx::FromRow)]
    struct CityCount {
        city: Option<String>,
        count: Option<i64>,
    }

    let city_counts: Vec<CityCount> = sqlx::query_as(
        "SELECT city, COUNT(*) as count FROM places WHERE is_active = true GROUP BY city ORDER BY count DESC LIMIT 10"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

    // Query average rating
    let avg_rating: (Option<f32>,) = sqlx::query_as(
        "SELECT AVG(google_rating) FROM places WHERE is_active = true AND google_rating IS NOT NULL"
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "places_by_type": type_counts,
        "places_by_city": city_counts,
        "average_rating": avg_rating.0,
    })))
}

/// Helper function to verify admin authentication
/// DOCUMENTATION: Checks X-Admin-Token header against configured admin token
fn verify_admin_token(req: &HttpRequest, config: &Config) -> Result<(), PlacesError> {
    let token = req
        .headers()
        .get("X-Admin-Token")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            log::warn!("Admin request without token");
            PlacesError::Unauthorized
        })?;

    if token != config.admin_token {
        log::warn!("Admin request with invalid token");
        return Err(PlacesError::Forbidden);
    }

    Ok(())
}

/// GET /admin/places/{id}/raw
/// Get raw place data for debugging
///
/// DOCUMENTATION: Returns both database record and Google Places data
/// Useful for debugging mapping issues
pub async fn get_place_raw(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
    path: web::Path<uuid::Uuid>,
) -> Result<impl Responder, PlacesError> {
    // Authenticate admin request
    verify_admin_token(&req, &config)?;

    let place_id = path.into_inner();

    // Query the database record with all fields (including internal ones)
    #[derive(Debug, Serialize, sqlx::FromRow)]
    struct RawPlaceRecord {
        id: uuid::Uuid,
        name: String,
        description: Option<String>,
        #[sqlx(rename = "type")]
        type_field: String,
        #[sqlx(skip)]
        longitude: f64,
        #[sqlx(skip)]
        latitude: f64,
        address: Option<String>,
        city: String,
        district: Option<String>,
        postal_code: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        website: Option<String>,
        google_place_id: Option<String>,
        google_place_url: Option<String>,
        google_rating: Option<f32>,
        google_rating_count: Option<i32>,
        price_level: Option<i32>,
        main_categories: Option<Vec<String>>,
        secondary_categories: Option<Vec<String>>,
        cuisine_types: Option<Vec<String>>,
        tags: Option<serde_json::Value>,
        vibe_descriptor: Option<serde_json::Value>,
        suitable_for: Option<Vec<String>>,
        opening_hours: Option<serde_json::Value>,
        is_open_now: Option<bool>,
        is_subscribed: Option<bool>,
        subscription_tier: Option<String>,
        business_status: Option<String>,
        is_active: Option<bool>,
        #[sqlx(default)]
        search_vector: Option<String>,
    }

    let record: RawPlaceRecord = sqlx::query_as(
        r#"
        SELECT 
            id, name, description, type,
            ST_X(location) as longitude, ST_Y(location) as latitude,
            address, city, district, postal_code,
            phone, email, website,
            google_place_id, google_place_url,
            google_rating, google_rating_count, price_level,
            main_categories, secondary_categories, cuisine_types,
            tags, vibe_descriptor, suitable_for,
            opening_hours, is_open_now,
            is_subscribed, subscription_tier,
            business_status, is_active,
            search_vector::text as search_vector
        FROM places
        WHERE id = $1
        "#,
    )
    .bind(place_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| PlacesError::DatabaseError(e.to_string()))?
    .ok_or_else(|| PlacesError::NotFound(place_id.to_string()))?;

    // If we have a google_place_id, try to fetch fresh data from Google
    let mut google_data = None;
    if let Some(ref gp_id) = record.google_place_id {
        if !config.google_places_api_key.is_empty() {
            let google_client = GooglePlacesClient::new(config.google_places_api_key.clone());
            
            match google_client.get_place_details(gp_id).await {
                Ok(place) => {
                    google_data = Some(place);
                }
                Err(e) => {
                    log::warn!("Could not fetch Google data for {}: {}", gp_id, e);
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "place_id": place_id,
        "database_record": record,
        "google_data": google_data,
        "note": "This endpoint exposes internal fields for debugging. Do not use in production API."
    })))
}

/// Configuration for admin routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/sync/{city}", web::post().to(sync_city))
            .route("/sync/batch", web::post().to(sync_cities_batch))
            .route("/sync/status", web::get().to(sync_status))
            .route("/stats", web::get().to(database_stats))
            .route("/places/{id}/raw", web::get().to(get_place_raw)),
    );
}
