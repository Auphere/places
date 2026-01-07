// src/services/place_service.rs
// DOCUMENTATION: Business logic for places
// PURPOSE: Intermediary between handlers and repository, handles extra logic

use crate::db::{PhotoRepository, PlaceRepository, ReviewRepository, TipRepository};
use crate::errors::PlacesError;
use crate::models::{
    ClusterQuery, ClusterResponse, CreatePlaceRequest, Place, PlaceDetailResponse, PlaceResponse, SearchQuery, SearchResponse,
    UpdatePlaceRequest, FrontendPlaceResponse, FrontendSearchResponse, FrontendCustomAttributes,
    FrontendPhotoResponse, FrontendReviewResponse, CreateTipRequest,
};
use crate::services::{GooglePlacesClient, FoursquareClient};
use crate::services::google_places_client::{GooglePlace, GooglePhoto, GoogleReview};
use crate::models::{CreatePhotoRequest, CreateReviewRequest};
use chrono::{TimeZone, Utc};
use std::time::Duration;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;

pub struct PlaceService;

impl PlaceService {
    /// Create a new place
    pub async fn create_place(
        pool: &PgPool,
        req: CreatePlaceRequest,
    ) -> Result<PlaceResponse, PlacesError> {
        // Here we could add extra validation logic, e.g. checking blacklist, etc.
        let place = PlaceRepository::create_place(pool, &req).await?;
        Ok(place.to_response())
    }

    /// Get a place by ID (UUID or Google Place ID) with optional on-demand Google enrichment.
    ///
    /// If `google_client` is provided and the record is stale (or missing photos/reviews),
    /// it will fetch Google Place Details and persist the updated place + photos + reviews.
    pub async fn get_place_by_id_or_google_id(
        pool: &PgPool,
        identifier: &str,
        google_client: Option<&GooglePlacesClient>,
        fsq_client: Option<&FoursquareClient>,
    ) -> Result<PlaceDetailResponse, PlacesError> {
        // Try to parse as UUID first
        let mut place = if let Ok(uuid) = Uuid::parse_str(identifier) {
            PlaceRepository::get_by_id(pool, uuid).await?
        } else {
            // If not a UUID, treat as Google Place ID.
            // If missing in DB, we can create it on-demand from Google Place Details.
            match PlaceRepository::get_by_google_place_id(pool, identifier).await {
                Ok(p) => p,
                Err(PlacesError::NotFound(_)) => {
                    let Some(client) = google_client else {
                        return Err(PlacesError::NotFound(identifier.to_string()));
                    };

                    log::info!("Place not found in DB, fetching from Google Place Details: {}", identifier);
                    let details = client.get_place_details(identifier).await?;
                    let inferred_city = Self::extract_city_from_google_place(&details)
                        .unwrap_or_else(|| "Unknown".to_string());
                    let create_req = client.to_create_request(&details, &inferred_city);
                    let (created, _) = PlaceRepository::upsert_google_place(pool, &create_req).await?;
                    created
                }
                Err(e) => return Err(e),
            }
        };
        
        let mut photos = PhotoRepository::get_photos_by_place(pool, &place.id, None).await?;
        let mut reviews = ReviewRepository::get_reviews_by_place(pool, &place.id, None).await?;
        let mut tips = TipRepository::get_tips_by_place(pool, &place.id, None).await?;

        // On-demand enrichment from Google (SWR-lite): if missing data or stale, refresh now.
        if let (Some(client), Some(ref google_place_id)) = (google_client, place.google_place_id.clone()) {
            let is_stale = place
                .last_verified_at
                .map(|ts| Utc::now().signed_duration_since(ts).to_std().unwrap_or(Duration::from_secs(0)) > Duration::from_secs(7 * 24 * 3600))
                .unwrap_or(true);
            let is_missing_assets = photos.is_empty() || reviews.is_empty();

            if is_stale || is_missing_assets {
                log::info!(
                    "Refreshing place from Google (stale={}, missing_assets={}): {}",
                    is_stale,
                    is_missing_assets,
                    google_place_id
                );
                if let Ok(details) = client.get_place_details(google_place_id).await {
                    // Upsert place core fields
                    let create_req = client.to_create_request(&details, &place.city);
                    let (updated_place, _) = PlaceRepository::upsert_google_place(pool, &create_req).await?;
                    place = updated_place;

                    // Persist reviews/photos (best-effort)
                    Self::persist_google_reviews(pool, &place.id, &details).await;
                    Self::persist_google_photos(pool, &place.id, &details, client).await;

                    // Reload place to get primary_photo_url from LEFT JOIN LATERAL
                    place = PlaceRepository::get_by_id(pool, place.id).await?;
                    
                    // Reload assets after refresh
                    photos = PhotoRepository::get_photos_by_place(pool, &place.id, None).await?;
                    reviews = ReviewRepository::get_reviews_by_place(pool, &place.id, None).await?;
                    tips = TipRepository::get_tips_by_place(pool, &place.id, None).await?;
                }
            }
        }

        // Optional Foursquare enrichment (photos + mapping stored in tags)
        if let Some(fsq_client) = fsq_client {
            if let Ok(Some(updated_tags)) = Self::enrich_place_with_foursquare(
                pool,
                &place.id,
                &place.name,
                place.latitude,
                place.longitude,
                place.tags.clone(),
                fsq_client,
            )
            .await
            {
                // Only write if tags changed (avoid extra writes)
                let should_update = match &place.tags {
                    Some(existing) => existing != &updated_tags,
                    None => true,
                };
                if should_update {
                    let updated_place = PlaceRepository::update_place(
                        pool,
                        place.id,
                        &UpdatePlaceRequest {
                            name: None,
                            description: None,
                            tags: Some(updated_tags),
                            vibe_descriptor: None,
                            opening_hours: None,
                            google_rating: None,
                            business_status: None,
                        },
                    )
                    .await?;
                    place = updated_place;
                }

                // Reload place to get updated primary_photo_url (may include Foursquare photos)
                place = PlaceRepository::get_by_id(pool, place.id).await?;
                
                // Reload photos to include foursquare ones
                photos = PhotoRepository::get_photos_by_place(pool, &place.id, None).await?;
                tips = TipRepository::get_tips_by_place(pool, &place.id, None).await?;
            }
        }

        Ok(PlaceDetailResponse {
            place: place.to_response(),
            photos: photos.into_iter().map(|p| p.to_response()).collect(),
            reviews: reviews.into_iter().map(|r| r.to_response()).collect(),
            tips: tips.into_iter().map(|t| t.to_response()).collect(),
        })
    }

    fn extract_city_from_google_place(google_place: &GooglePlace) -> Option<String> {
        let components = google_place.address_components.as_ref()?;
        for component in components {
            if component
                .types
                .iter()
                .any(|t| t == "locality" || t == "administrative_area_level_2")
            {
                return Some(component.long_name.clone());
            }
        }
        None
    }

    /// Optional Foursquare enrichment (photos + basic metadata).
    ///
    /// We store `fsq_id` inside `places.tags.external_ids.fsq_id` to avoid a schema migration.
    /// If you prefer a dedicated DB column for `fsq_id`, tell me and we’ll migrate it.
    pub async fn enrich_place_with_foursquare(
        pool: &PgPool,
        place_id: &Uuid,
        place_name: &str,
        lat: f64,
        lon: f64,
        existing_tags: Option<Value>,
        fsq_client: &FoursquareClient,
    ) -> Result<Option<Value>, PlacesError> {
        let mut tags = existing_tags.unwrap_or_else(|| serde_json::json!({}));

        // Read existing fsq_id if already stored.
        let existing_fsq_id = tags
            .get("external_ids")
            .and_then(|v| v.get("fsq_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let fsq_id = if let Some(id) = existing_fsq_id {
            id
        } else {
            // Attempt to match near the coordinate.
            let best = fsq_client.match_place(place_name, lat, lon, 200).await?;
            let Some(best) = best else { return Ok(Some(tags)); };
            let id = best.fsq_id;

            // Persist mapping in tags (best-effort)
            tags["external_ids"]["fsq_id"] = serde_json::Value::String(id.clone());
            tags["external_ids"]["fsq_name"] = serde_json::Value::String(best.name);
            if let Some(rating) = best.rating {
                tags["foursquare"]["rating_0_10"] = serde_json::Value::Number(
                    serde_json::Number::from_f64(rating as f64).unwrap_or_else(|| serde_json::Number::from(0)),
                );
            }
            if let Some(popularity) = best.popularity {
                tags["foursquare"]["popularity"] = serde_json::Value::Number(
                    serde_json::Number::from_f64(popularity as f64).unwrap_or_else(|| serde_json::Number::from(0)),
                );
            }
            id
        };

        // Fetch and persist photos (best-effort).
        let photos = fsq_client.get_photos(&fsq_id, 10).await.unwrap_or_default();
        for (idx, photo) in photos.iter().enumerate() {
            let photo_url = FoursquareClient::photo_url(&photo.prefix, "800x800", &photo.suffix);
            let thumbnail_url = Some(FoursquareClient::photo_url(&photo.prefix, "300x300", &photo.suffix));
            let photo_req = CreatePhotoRequest {
                place_id: *place_id,
                source: "foursquare".to_string(),
                source_photo_reference: Some(photo.id.clone()),
                photo_url,
                thumbnail_url,
                width: photo.width,
                height: photo.height,
                attribution: Some("Foursquare".to_string()),
                is_primary: Some(false),
                display_order: Some(idx as i32),
            };
            if let Err(e) = PhotoRepository::create_photo(pool, &photo_req).await {
                log::debug!("Failed to upsert foursquare photo (best-effort): {}", e);
            }
        }

        // Fetch and persist tips (best-effort).
        // Tips do not have rating → stored in place_tips.
        let tips = fsq_client.get_tips(&fsq_id, 10).await.unwrap_or_default();
        for tip in tips {
            let like_count = tip
                .agree_count
                .unwrap_or(0)
                .saturating_sub(tip.disagree_count.unwrap_or(0));
            let posted_at = tip
                .created_at
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let tip_req = CreateTipRequest {
                place_id: *place_id,
                source: "foursquare".to_string(),
                source_id: Some(tip.id),
                author: None,
                text: tip.text,
                posted_at,
                like_count: Some(like_count),
            };
            if let Err(e) = TipRepository::create_tip(pool, &tip_req).await {
                log::debug!("Failed to upsert foursquare tip (best-effort): {}", e);
            }
        }

        Ok(Some(tags))
    }

    async fn persist_google_reviews(pool: &PgPool, place_id: &Uuid, details: &GooglePlace) {
        let Some(ref reviews) = details.reviews else { return; };
        for review in reviews {
            let Some(rating) = review.rating else { continue; };
            let review_req = CreateReviewRequest {
                place_id: *place_id,
                source: "google".to_string(),
                source_id: Some(format!(
                    "{}_{}",
                    details.place_id,
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
                has_photo: review.profile_photo_url.is_some().then_some(true),
            };
            if let Err(e) = ReviewRepository::create_review(pool, &review_req).await {
                log::debug!("Failed to upsert google review (best-effort): {}", e);
            }
        }
    }

    async fn persist_google_photos(pool: &PgPool, place_id: &Uuid, details: &GooglePlace, client: &GooglePlacesClient) {
        let Some(ref photos) = details.photos else { return; };
        for (idx, photo) in photos.iter().enumerate() {
            let photo_req = CreatePhotoRequest {
                place_id: *place_id,
                source: "google".to_string(),
                source_photo_reference: Some(photo.photo_reference.clone()),
                photo_url: client.get_photo_url(&photo.photo_reference, Some(800)),
                thumbnail_url: Some(client.get_photo_thumbnail_url(&photo.photo_reference)),
                width: photo.width,
                height: photo.height,
                attribution: photo.html_attributions.as_ref().and_then(|attrs| attrs.first().cloned()),
                is_primary: Some(idx == 0),
                display_order: Some(idx as i32),
            };
            if let Err(e) = PhotoRepository::create_photo(pool, &photo_req).await {
                log::debug!("Failed to upsert google photo (best-effort): {}", e);
            }
        }
    }

    /// Search for places (from database)
    pub async fn search_places(
        pool: &PgPool,
        query: SearchQuery,
    ) -> Result<SearchResponse, PlacesError> {
        let (places, total_count) = PlaceRepository::search(pool, &query).await?;

        // Calculate pagination metadata
        let limit = query.limit.unwrap_or(20).max(1);
        let page = query.page.unwrap_or(1).max(1);
        let has_more = total_count > (page * limit);

        Ok(SearchResponse {
            data: places.iter().map(|p| p.to_response()).collect(),
            total_count,
            page,
            limit,
            has_more,
        })
    }

    /// Cluster places using DBSCAN (PostGIS).
    pub async fn cluster_places(pool: &PgPool, query: ClusterQuery) -> Result<ClusterResponse, PlacesError> {
        PlaceRepository::cluster_places(pool, &query).await
    }

    /// Search places directly from Google Places API
    /// DOCUMENTATION: Fetches places from Google Places API and transforms to frontend format
    pub async fn search_places_from_google(
        google_client: &GooglePlacesClient,
        query: SearchQuery,
    ) -> Result<FrontendSearchResponse, PlacesError> {
        // Extract search parameters
        let latitude = query.lat;
        let longitude = query.lon;
        let radius_meters = query.radius_km.map(|km| (km * 1000.0) as u32).unwrap_or(5000);
        let place_type = query.type_.as_deref();
        let keyword = query.q.as_deref().unwrap_or("").trim();

        // If we have coordinates, use Nearby Search (best for proximity).
        // Otherwise, fall back to Text Search using city + query.
        let google_places = match (latitude, longitude) {
            (Some(lat), Some(lon)) => {
                google_client
                    .nearby_search(lat, lon, radius_meters, place_type, if keyword.is_empty() { None } else { Some(keyword) })
                    .await?
            }
            _ => {
                // Build a text query. Prefer: "{keyword} in {city}".
                // If keyword is empty, we still allow city-only search (e.g., "restaurants in Madrid")
                // by using place_type as a weak fallback.
                let city = query.city.clone().unwrap_or_default();
                let inferred = if !keyword.is_empty() {
                    if city.is_empty() { keyword.to_string() } else { format!("{} in {}", keyword, city) }
                } else if let Some(t) = place_type {
                    if city.is_empty() { t.to_string() } else { format!("{} in {}", t, city) }
                } else {
                return Err(PlacesError::ValidationError(
                        "Provide either (lat & lon) or (q/city) for search".to_string(),
                    ));
                };

                google_client
                    .text_search(
                        &inferred,
                        None,
                        Some(radius_meters),
                        place_type,
                    )
                    .await?
            }
        };

        // Transform places to frontend format
        // ⚠️ OPTIMIZATION: Removed get_place_details call to reduce API usage by 50%
        // The nearby_search already provides sufficient data for listing
        // Details (photos, reviews) are fetched only when user clicks on a place
        let mut frontend_places = Vec::new();
        for google_place in google_places.iter() {
            // Transform to frontend format using data from nearby_search
            let frontend_place = Self::transform_google_place_to_frontend(
                &google_place,
                google_client,
                latitude,
                longitude,
                query.city.as_deref(),
            )?;
            frontend_places.push(frontend_place);
        }

        // Calculate pagination
        let per_page = query.limit.unwrap_or(20).max(1).min(100);
        let page = query.page.unwrap_or(1).max(1);
        let total = frontend_places.len() as i64;
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;

        // Apply pagination
        let start = ((page - 1) * per_page) as usize;
        let end = (start + per_page as usize).min(frontend_places.len());
        let paginated_places = if start < frontend_places.len() {
            frontend_places[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(FrontendSearchResponse {
            places: paginated_places,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Transform Google Place to Frontend format
    fn transform_google_place_to_frontend(
        google_place: &GooglePlace,
        google_client: &GooglePlacesClient,
        search_lat: Option<f64>,
        search_lon: Option<f64>,
        city: Option<&str>,
    ) -> Result<FrontendPlaceResponse, PlacesError> {
        // Extract city and district from address components
        let (city_name, district) = Self::extract_city_and_district(
            &google_place.address_components,
            city,
        );

        // Transform photos
        let photos: Vec<FrontendPhotoResponse> = google_place
            .photos
            .as_ref()
            .map(|photo_list| {
                photo_list
                    .iter()
                    .map(|photo| Self::transform_google_photo(photo, google_client))
                    .collect()
            })
            .unwrap_or_default();

        // Get primary photo URLs
        let primary_photo_url = photos.first().map(|p| p.photo_url.clone());
        let primary_photo_thumbnail_url = photos.first().and_then(|p| p.thumbnail_url.clone());

        // Transform reviews
        let reviews = google_place
            .reviews
            .as_ref()
            .map(|review_list| {
                review_list
                    .iter()
                    .map(Self::transform_google_review)
                    .collect()
            })
            .unwrap_or_default();

        // Calculate distance if search coordinates provided
        let distance_km = match (search_lat, search_lon) {
            (Some(search_lat), Some(search_lon)) => {
                Some(Self::calculate_distance(
                    search_lat,
                    search_lon,
                    google_place.geometry.location.lat,
                    google_place.geometry.location.lng,
                ))
            }
            _ => None,
        };

        Ok(FrontendPlaceResponse {
            place_id: google_place.place_id.clone(),
            name: google_place.name.clone(),
            formatted_address: google_place.formatted_address.clone(),
            vicinity: google_place.vicinity.clone(),
            latitude: google_place.geometry.location.lat,
            longitude: google_place.geometry.location.lng,
            types: google_place.types.clone(),
            rating: google_place.rating,
            user_ratings_total: google_place.user_ratings_total,
            price_level: google_place.price_level,
            phone_number: google_place
                .formatted_phone_number
                .clone()
                .or_else(|| google_place.international_phone_number.clone()),
            website: google_place.website.clone(),
            opening_hours: google_place
                .opening_hours
                .as_ref()
                .and_then(|hours| serde_json::to_value(hours).ok()),
            is_open: google_place
                .opening_hours
                .as_ref()
                .and_then(|hours| hours.open_now),
            distance_km,
            custom_attributes: FrontendCustomAttributes {
                city: city_name,
                district,
                primary_photo_url,
                primary_photo_thumbnail_url,
                google_place_id: google_place.place_id.clone(),
                photos,
                reviews,
            },
        })
    }

    /// Extract city and district from address components
    fn extract_city_and_district(
        address_components: &Option<Vec<crate::services::google_places_client::GoogleAddressComponent>>,
        fallback_city: Option<&str>,
    ) -> (Option<String>, Option<String>) {
        let mut city = None;
        let mut district = None;

        if let Some(components) = address_components {
            for component in components {
                // Extract city
                if component.types.iter().any(|t| t == "locality" || t == "administrative_area_level_2") {
                    if city.is_none() {
                        city = Some(component.long_name.clone());
                    }
                }
                
                // Extract district/neighborhood
                if component.types.iter().any(|t| {
                    t == "sublocality"
                        || t == "sublocality_level_1"
                        || t == "neighborhood"
                        || t == "administrative_area_level_3"
                }) {
                    if district.is_none() {
                        district = Some(component.long_name.clone());
                    }
                }
            }
        }

        // Use fallback city if not found in address components
        let city = city.or_else(|| fallback_city.map(|s| s.to_string()));

        (city, district)
    }

    /// Transform Google Photo to Frontend format
    fn transform_google_photo(
        photo: &GooglePhoto,
        google_client: &GooglePlacesClient,
    ) -> FrontendPhotoResponse {
        let photo_url = google_client.get_photo_url(&photo.photo_reference, Some(800));
        let thumbnail_url = Some(google_client.get_photo_thumbnail_url(&photo.photo_reference));

        FrontendPhotoResponse {
            photo_url,
            thumbnail_url,
            width: photo.width,
            height: photo.height,
            attribution: photo.html_attributions.as_ref().and_then(|attrs| {
                attrs.first().cloned()
            }),
        }
    }

    /// Transform Google Review to Frontend format
    fn transform_google_review(review: &GoogleReview) -> FrontendReviewResponse {
        FrontendReviewResponse {
            author: review.author_name.clone(),
            rating: review.rating,
            text: review.text.clone(),
            relative_time_description: review.relative_time_description.clone(),
        }
    }

    /// Calculate distance between two coordinates in kilometers
    /// Uses Haversine formula
    fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + (lat1.to_radians().cos())
                * (lat2.to_radians().cos())
                * (d_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().asin();

        EARTH_RADIUS_KM * c
    }

    /// Update a place
    pub async fn update_place(
        pool: &PgPool,
        id: Uuid,
        req: UpdatePlaceRequest,
    ) -> Result<PlaceResponse, PlacesError> {
        let place = PlaceRepository::update_place(pool, id, &req).await?;
        Ok(place.to_response())
    }

    /// Delete a place
    pub async fn delete_place(pool: &PgPool, id: Uuid) -> Result<(), PlacesError> {
        PlaceRepository::delete_place(pool, id).await
    }

    /// Upsert a place (create or update based on google_place_id)
    pub async fn upsert_place(
        pool: &PgPool,
        req: CreatePlaceRequest,
    ) -> Result<(Place, bool), PlacesError> {
        PlaceRepository::upsert_google_place(pool, &req).await
    }
}
