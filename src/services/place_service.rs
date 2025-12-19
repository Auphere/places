// src/services/place_service.rs
// DOCUMENTATION: Business logic for places
// PURPOSE: Intermediary between handlers and repository, handles extra logic

use crate::db::{PhotoRepository, PlaceRepository, ReviewRepository};
use crate::errors::PlacesError;
use crate::models::{
    CreatePlaceRequest, Place, PlaceDetailResponse, PlaceResponse, SearchQuery, SearchResponse,
    UpdatePlaceRequest, FrontendPlaceResponse, FrontendSearchResponse, FrontendCustomAttributes,
    FrontendPhotoResponse, FrontendReviewResponse,
};
use crate::services::GooglePlacesClient;
use crate::services::google_places_client::{GooglePlace, GooglePhoto, GoogleReview};
use sqlx::PgPool;
use uuid::Uuid;

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

    /// Get a place by ID (UUID or Google Place ID)
    pub async fn get_place_by_id_or_google_id(
        pool: &PgPool,
        identifier: &str,
    ) -> Result<PlaceDetailResponse, PlacesError> {
        // Try to parse as UUID first
        let place = if let Ok(uuid) = Uuid::parse_str(identifier) {
            PlaceRepository::get_by_id(pool, uuid).await?
        } else {
            // If not a UUID, treat as Google Place ID
            PlaceRepository::get_by_google_place_id(pool, identifier).await?
        };
        
        let photos = PhotoRepository::get_photos_by_place(pool, &place.id, None).await?;
        let reviews = ReviewRepository::get_reviews_by_place(pool, &place.id, None).await?;

        Ok(PlaceDetailResponse {
            place: place.to_response(),
            photos: photos.into_iter().map(|p| p.to_response()).collect(),
            reviews: reviews.into_iter().map(|r| r.to_response()).collect(),
        })
    }

    /// Get a place by ID (UUID only)
    pub async fn get_place(pool: &PgPool, id: Uuid) -> Result<PlaceDetailResponse, PlacesError> {
        let place = PlaceRepository::get_by_id(pool, id).await?;
        let photos = PhotoRepository::get_photos_by_place(pool, &place.id, None).await?;
        let reviews = ReviewRepository::get_reviews_by_place(pool, &place.id, None).await?;

        Ok(PlaceDetailResponse {
            place: place.to_response(),
            photos: photos.into_iter().map(|p| p.to_response()).collect(),
            reviews: reviews.into_iter().map(|r| r.to_response()).collect(),
        })
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
        let keyword = query.q.as_deref();
        
        // Validate that we have coordinates for nearby search
        let (lat, lon) = match (latitude, longitude) {
            (Some(lat), Some(lon)) => (lat, lon),
            _ => {
                // If no coordinates, try to use city name for text search
                // For now, return error - we need coordinates for nearby search
                return Err(PlacesError::ValidationError(
                    "Latitude and longitude are required for search".to_string()
                ));
            }
        };

        // Perform nearby search
        let google_places = google_client
            .nearby_search(lat, lon, radius_meters, place_type, keyword)
            .await?;

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
