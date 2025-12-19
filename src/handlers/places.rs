// src/handlers/places.rs
// DOCUMENTATION: HTTP handlers for place operations
// PURPOSE: Parse requests, call services, return responses

use crate::config::Config;
use crate::errors::PlacesError;
use crate::models::{CreatePlaceRequest, SearchQuery, UpdatePlaceRequest};
use crate::services::{PlaceService, GooglePlacesClient, PlacesCache};
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// POST /places
/// Create a new place
pub async fn create_place(
    pool: web::Data<PgPool>,
    req: web::Json<CreatePlaceRequest>,
) -> Result<impl Responder, PlacesError> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(PlacesError::ValidationError(e.to_string()));
    }

    let place = PlaceService::create_place(pool.get_ref(), req.into_inner()).await?;
    Ok(HttpResponse::Created().json(place))
}

/// POST /places/upsert
/// Create or update a place based on google_place_id
pub async fn upsert_place(
    pool: web::Data<PgPool>,
    req: web::Json<CreatePlaceRequest>,
) -> Result<impl Responder, PlacesError> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(PlacesError::ValidationError(e.to_string()));
    }

    let (place, created) = PlaceService::upsert_place(pool.get_ref(), req.into_inner()).await?;
    
    if created {
        Ok(HttpResponse::Created().json(place.to_response()))
    } else {
        Ok(HttpResponse::Ok().json(place.to_response()))
    }
}

/// GET /places/{id}
/// Retrieve a place by ID (UUID or Google Place ID)
pub async fn get_place(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, PlacesError> {
    let identifier = path.into_inner();
    let place = PlaceService::get_place_by_id_or_google_id(pool.get_ref(), &identifier).await?;
    Ok(HttpResponse::Ok().json(place))
}

/// GET /places/search
/// Search places with filters (from Google Places API with caching)
pub async fn search_places(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    cache: web::Data<Arc<PlacesCache>>,
    query: web::Query<SearchQuery>,
) -> Result<impl Responder, PlacesError> {
    // Check if Google Places API key is configured
    if config.google_places_api_key.is_empty() {
        // Fallback to database search if API key not configured
        let result = PlaceService::search_places(pool.get_ref(), query.into_inner()).await?;
        return Ok(HttpResponse::Ok().json(result));
    }

    // Use Google Places API directly with shared cache
    let google_client = GooglePlacesClient::new_with_cache(
        config.google_places_api_key.clone(),
        cache.get_ref().clone()
    );
    let result = PlaceService::search_places_from_google(&google_client, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// PUT /places/{id}
/// Update a place
pub async fn update_place(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<UpdatePlaceRequest>,
) -> Result<impl Responder, PlacesError> {
    let place =
        PlaceService::update_place(pool.get_ref(), path.into_inner(), req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(place))
}

/// DELETE /places/{id}
/// Soft delete a place
pub async fn delete_place(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, PlacesError> {
    PlaceService::delete_place(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Configuration for place routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/places")
            .route("", web::post().to(create_place))
            .route("/upsert", web::post().to(upsert_place))
            .route("/search", web::get().to(search_places))
            .route("/{id}", web::get().to(get_place))
            .route("/{id}", web::put().to(update_place))
            .route("/{id}", web::delete().to(delete_place)),
    );
}
