// src/services/sync_service.rs
// DOCUMENTATION: Google Places synchronization service
// PURPOSE: Orchestrate bulk data import from Google Places API

use crate::db::{PhotoRepository, PlaceRepository, ReviewRepository};
use crate::errors::PlacesError;
use crate::models::{CreatePhotoRequest, CreateReviewRequest};
use crate::services::{GooglePlacesClient, GridGenerator};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

/// Synchronization statistics
/// DOCUMENTATION: Tracks results of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    /// City that was synced
    pub city: String,
    /// Total number of API requests made
    pub api_requests: u32,
    /// Total places retrieved from API
    pub places_retrieved: u32,
    /// Places successfully created in database
    pub places_created: u32,
    /// Places skipped (already exist)
    pub places_skipped: u32,
    /// Places that failed to insert
    pub places_failed: u32,
    /// Reviews successfully created
    pub reviews_created: u32,
    /// Photos successfully created
    pub photos_created: u32,
    /// Error messages encountered
    pub errors: Vec<String>,
    /// Total sync duration in seconds
    pub duration_seconds: u64,
    /// Timestamp when sync started
    pub started_at: String,
    /// Timestamp when sync completed
    pub completed_at: Option<String>,
}

impl SyncStats {
    /// Create new sync statistics tracker
    pub fn new(city: String) -> Self {
        Self {
            city,
            api_requests: 0,
            places_retrieved: 0,
            places_created: 0,
            places_skipped: 0,
            places_failed: 0,
            reviews_created: 0,
            photos_created: 0,
            errors: Vec::new(),
            duration_seconds: 0,
            started_at: Utc::now().to_rfc3339(),
            completed_at: None,
        }
    }

    /// Mark sync as completed
    pub fn complete(&mut self, duration: u64) {
        self.duration_seconds = duration;
        self.completed_at = Some(Utc::now().to_rfc3339());
    }
}

/// Sync service for Google Places integration
/// DOCUMENTATION: Handles bulk synchronization of places from Google Places API
pub struct SyncService;

impl SyncService {
    /// Synchronize places for a city
    /// DOCUMENTATION: Main sync method - generates grid and fetches places for each cell
    ///
    /// Process:
    /// 1. Generate geographic grid for city
    /// 2. For each grid cell, query Google Places API
    /// 3. Convert Google places to internal format
    /// 4. Check for duplicates (by google_place_id)
    /// 5. Insert new places into database
    /// 6. Return statistics
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `google_client` - Google Places API client
    /// * `city` - City name to sync
    /// * `place_type` - Optional filter (e.g., "restaurant", "bar")
    /// * `cell_size_km` - Grid cell size (default 1.5 km)
    /// * `radius_m` - Search radius per cell (default 1000 m)
    ///
    /// # Returns
    /// SyncStats with operation results
    pub async fn sync_city(
        pool: &PgPool,
        google_client: &GooglePlacesClient,
        city: &str,
        place_type: Option<&str>,
        cell_size_km: Option<f64>,
        radius_m: Option<u32>,
    ) -> Result<SyncStats, PlacesError> {
        let start_time = Instant::now();
        let mut stats = SyncStats::new(city.to_string());

        log::info!("Starting sync for city: {}", city);

        // Generate grid cells for the city
        let cells =
            GridGenerator::generate_for_city(city, cell_size_km, radius_m).map_err(|e| {
                log::error!("Failed to generate grid for {}: {}", city, e);
                PlacesError::InvalidInput(e)
            })?;

        log::info!("Generated {} grid cells for {}", cells.len(), city);

        // Process each grid cell
        for (idx, cell) in cells.iter().enumerate() {
            log::debug!(
                "Processing cell {}/{}: lat={}, lng={}, radius={}",
                idx + 1,
                cells.len(),
                cell.latitude,
                cell.longitude,
                cell.radius
            );

            // Query Google Places API for this cell
            match google_client
                .nearby_search(cell.latitude, cell.longitude, cell.radius, place_type, None)
                .await
            {
                Ok(google_places) => {
                    stats.api_requests += 1;
                    stats.places_retrieved += google_places.len() as u32;

                    log::info!(
                        "Cell {}/{}: Retrieved {} places",
                        idx + 1,
                        cells.len(),
                        google_places.len()
                    );

                    // Process each place from this cell
                    for google_place in google_places {
                        // Fetch detailed information from Place Details API
                        log::debug!("Fetching details for: {}", google_place.name);
                        let detailed_place = match google_client
                            .get_place_details(&google_place.place_id)
                            .await
                        {
                            Ok(details) => {
                                stats.api_requests += 1; // Count Place Details API call
                                details
                            }
                            Err(e) => {
                                log::warn!(
                                    "Could not fetch details for {}: {}. Using basic info.",
                                    google_place.name,
                                    e
                                );
                                // If details fetch fails, use the basic info from nearby search
                                google_place
                            }
                        };

                        // Convert to CreatePlaceRequest (now with full details)
                        let create_req = google_client.to_create_request(&detailed_place, city);

                        // Upsert into database
                        let (place, created) =
                            match PlaceRepository::upsert_google_place(pool, &create_req).await {
                                Ok(result) => result,
                                Err(e) => {
                                    stats.places_failed += 1;
                                    let error_msg =
                                        format!("Failed to store {}: {}", create_req.name, e);
                                    log::warn!("{}", error_msg);
                                    stats.errors.push(error_msg);
                                    continue;
                                }
                            };

                        if created {
                            stats.places_created += 1;
                        } else {
                            stats.places_skipped += 1;
                        }

                        log::debug!("Upserted place: {}", create_req.name);

                        // Save reviews (if available)
                        if let Some(ref reviews) = detailed_place.reviews {
                            for review in reviews {
                                if let Some(rating) = review.rating {
                                    let review_req = CreateReviewRequest {
                                        place_id: place.id,
                                        source: "google".to_string(),
                                        source_id: Some(format!(
                                            "{}_{}",
                                            detailed_place.place_id,
                                            review.time.unwrap_or(0)
                                        )),
                                        author: review.author_name.clone(),
                                        rating: rating as f32,
                                        text: review.text.clone(),
                                        posted_at: review
                                            .time
                                            .and_then(|t| Utc.timestamp_opt(t, 0).single())
                                            .unwrap_or_else(|| Utc::now()),
                                        is_verified: Some(false),
                                        has_photo: review
                                            .profile_photo_url
                                            .is_some()
                                            .then_some(true),
                                    };

                                    match ReviewRepository::create_review(pool, &review_req).await {
                                        Ok(_) => {
                                            stats.reviews_created += 1;
                                        }
                                        Err(e) => {
                                            log::warn!(
                                                "Failed to save review for {}: {}",
                                                create_req.name,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        // Save photos (if available)
                        if let Some(ref photos) = detailed_place.photos {
                            for (idx, photo) in photos.iter().enumerate() {
                                // Construct Google Places Photo URL
                                let photo_url = format!(
                                    "https://maps.googleapis.com/maps/api/place/photo?maxwidth=800&photoreference={}&key={}",
                                    photo.photo_reference,
                                    google_client.get_api_key()
                                );

                                let thumbnail_url = format!(
                                    "https://maps.googleapis.com/maps/api/place/photo?maxwidth=400&photoreference={}&key={}",
                                    photo.photo_reference,
                                    google_client.get_api_key()
                                );

                                let photo_req = CreatePhotoRequest {
                                    place_id: place.id,
                                    source: "google".to_string(),
                                    source_photo_reference: Some(photo.photo_reference.clone()),
                                    photo_url,
                                    thumbnail_url: Some(thumbnail_url),
                                    width: photo.width,
                                    height: photo.height,
                                    attribution: photo
                                        .html_attributions
                                        .as_ref()
                                        .and_then(|attrs| attrs.first().cloned()),
                                    is_primary: Some(idx == 0), // First photo is primary
                                    display_order: Some(idx as i32),
                                };

                                match PhotoRepository::create_photo(pool, &photo_req).await {
                                    Ok(_) => {
                                        stats.photos_created += 1;
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to save photo for {}: {}",
                                            create_req.name,
                                            e
                                        );
                                    }
                                }
                            }
                        }

                        // Rate limiting: small delay between Place Details calls
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    let error_msg = format!("API error for cell {}: {}", cell.cell_id, e);
                    log::error!("{}", error_msg);
                    stats.errors.push(error_msg);

                    // Check if it's a rate limit error
                    if matches!(e, PlacesError::RateLimitExceeded) {
                        log::error!("Rate limit exceeded, stopping sync");
                        break;
                    }
                }
            }

            // Add small delay between requests to respect API rate limits
            // Google allows 100 req/sec, we'll be more conservative
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        let duration = start_time.elapsed().as_secs();
        stats.complete(duration);

        log::info!(
            "Sync completed for {}: {} created, {} skipped, {} failed in {}s",
            city,
            stats.places_created,
            stats.places_skipped,
            stats.places_failed,
            duration
        );

        Ok(stats)
    }

    /// Synchronize places for multiple cities
    /// DOCUMENTATION: Batch sync operation for multiple cities
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `google_client` - Google Places API client
    /// * `cities` - List of city names to sync
    /// * `place_type` - Optional place type filter
    ///
    /// # Returns
    /// Vector of SyncStats, one per city
    pub async fn sync_cities(
        pool: &PgPool,
        google_client: &GooglePlacesClient,
        cities: &[String],
        place_type: Option<&str>,
    ) -> Vec<SyncStats> {
        let mut all_stats = Vec::new();

        for city in cities {
            log::info!("Starting sync for city: {}", city);

            match Self::sync_city(pool, google_client, city, place_type, None, None).await {
                Ok(stats) => {
                    all_stats.push(stats);
                }
                Err(e) => {
                    log::error!("Failed to sync city {}: {}", city, e);

                    let mut stats = SyncStats::new(city.clone());
                    stats.errors.push(format!("Sync failed: {}", e));
                    stats.complete(0);
                    all_stats.push(stats);
                }
            }

            // Add delay between cities to avoid overwhelming the API
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        all_stats
    }

    /// Get sync summary across multiple city syncs
    /// DOCUMENTATION: Aggregates statistics from multiple sync operations
    ///
    /// # Arguments
    /// * `stats_list` - List of sync statistics from multiple operations
    ///
    /// # Returns
    /// Aggregated SyncStats representing the total
    pub fn aggregate_stats(stats_list: &[SyncStats]) -> SyncStats {
        let mut aggregated = SyncStats::new("Multiple Cities".to_string());

        for stats in stats_list {
            aggregated.api_requests += stats.api_requests;
            aggregated.places_retrieved += stats.places_retrieved;
            aggregated.places_created += stats.places_created;
            aggregated.places_skipped += stats.places_skipped;
            aggregated.places_failed += stats.places_failed;
            aggregated.duration_seconds += stats.duration_seconds;
            aggregated.errors.extend(stats.errors.clone());
        }

        aggregated.completed_at = Some(Utc::now().to_rfc3339());
        aggregated
    }

    /// Update existing places with fresh data from Google
    /// DOCUMENTATION: Refresh existing places with current Google data
    /// Future enhancement - not yet implemented
    #[allow(dead_code)]
    pub async fn refresh_existing_places(
        _pool: &PgPool,
        _google_client: &GooglePlacesClient,
    ) -> Result<SyncStats, PlacesError> {
        // TODO: Implement refresh logic
        // 1. Query all places with google_place_id
        // 2. For each place, call Google Places Details API
        // 3. Update rating, status, and other dynamic fields

        Err(PlacesError::InternalError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_stats_creation() {
        let stats = SyncStats::new("Madrid".to_string());

        assert_eq!(stats.city, "Madrid");
        assert_eq!(stats.places_created, 0);
        assert_eq!(stats.places_skipped, 0);
        assert_eq!(stats.places_failed, 0);
        assert!(stats.completed_at.is_none());
    }

    #[test]
    fn test_sync_stats_complete() {
        let mut stats = SyncStats::new("Barcelona".to_string());
        stats.places_created = 100;
        stats.places_skipped = 20;

        stats.complete(60);

        assert_eq!(stats.duration_seconds, 60);
        assert!(stats.completed_at.is_some());
    }

    #[test]
    fn test_aggregate_stats() {
        let stats1 = SyncStats {
            city: "Madrid".to_string(),
            api_requests: 10,
            places_retrieved: 100,
            places_created: 80,
            places_skipped: 15,
            places_failed: 5,
            errors: vec!["Error 1".to_string()],
            duration_seconds: 60,
            started_at: Utc::now().to_rfc3339(),
            completed_at: Some(Utc::now().to_rfc3339()),
        };

        let stats2 = SyncStats {
            city: "Barcelona".to_string(),
            api_requests: 8,
            places_retrieved: 80,
            places_created: 70,
            places_skipped: 8,
            places_failed: 2,
            errors: vec!["Error 2".to_string()],
            duration_seconds: 50,
            started_at: Utc::now().to_rfc3339(),
            completed_at: Some(Utc::now().to_rfc3339()),
        };

        let aggregated = SyncService::aggregate_stats(&[stats1, stats2]);

        assert_eq!(aggregated.api_requests, 18);
        assert_eq!(aggregated.places_retrieved, 180);
        assert_eq!(aggregated.places_created, 150);
        assert_eq!(aggregated.places_skipped, 23);
        assert_eq!(aggregated.places_failed, 7);
        assert_eq!(aggregated.duration_seconds, 110);
        assert_eq!(aggregated.errors.len(), 2);
    }
}
