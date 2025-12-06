// src/services/place_service.rs
// DOCUMENTATION: Business logic for places
// PURPOSE: Intermediary between handlers and repository, handles extra logic

use crate::db::{PhotoRepository, PlaceRepository, ReviewRepository};
use crate::errors::PlacesError;
use crate::models::{
    CreatePlaceRequest, Place, PlaceDetailResponse, PlaceResponse, SearchQuery, SearchResponse,
    UpdatePlaceRequest,
};
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

    /// Search for places
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
