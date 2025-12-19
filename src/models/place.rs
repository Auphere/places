// src/models/place.rs
// DOCUMENTATION: Core data structures for places
// PURPOSE: Defines all serialization/deserialization models for API and database

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use super::{PhotoResponse, ReviewResponse};

/// Represents a complete place record from the database
/// DOCUMENTATION: This struct maps directly to the places table in PostgreSQL
/// Used for internal operations and database queries
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Place {
    /// Unique identifier (UUID v4)
    pub id: Uuid,

    /// Place name - required field for all places
    pub name: String,

    /// Optional detailed description
    pub description: Option<String>,

    /// Place type: restaurant, bar, cafe, club, etc.
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_field: String,

    /// Geographic coordinates - longitude (extracted from PostGIS POINT)
    /// These will be populated from ST_X(location) and ST_Y(location) in queries
    #[sqlx(skip)]
    pub longitude: f64,

    /// Geographic coordinates - latitude (extracted from PostGIS POINT)
    #[sqlx(skip)]
    pub latitude: f64,

    /// Physical street address
    pub address: Option<String>,

    /// City name (required for filtering)
    pub city: String,

    /// Neighborhood or district
    pub district: Option<String>,

    /// Postal code
    pub postal_code: Option<String>,

    /// Phone number
    pub phone: Option<String>,

    /// Email address
    pub email: Option<String>,

    /// Website URL
    pub website: Option<String>,

    /// Google Places unique identifier (used for deduplication)
    pub google_place_id: Option<String>,

    /// Google Maps URL for this place
    pub google_place_url: Option<String>,

    /// Rating from Google (0-5)
    pub google_rating: Option<f32>,

    /// Number of ratings on Google
    pub google_rating_count: Option<i32>,

    /// Price level from Google (0-4: free to very expensive)
    pub price_level: Option<i32>,

    /// Main category tags
    pub main_categories: Option<Vec<String>>,

    /// Secondary category tags
    pub secondary_categories: Option<Vec<String>>,

    /// Cuisine types (italian, japanese, tapas, etc.) for restaurants
    pub cuisine_types: Option<Vec<String>>,

    /// Custom JSON metadata
    pub tags: Option<Value>,

    /// Atmosphere/vibe descriptor (music, crowd level, energy, etc.)
    pub vibe_descriptor: Option<Value>,

    /// Suitable for tags (couples, families, groups, solo, etc.)
    pub suitable_for: Option<Vec<String>>,

    /// Opening hours by day of week
    pub opening_hours: Option<Value>,

    /// Whether the place is currently open
    pub is_open_now: Option<bool>,

    /// B2B subscription status
    pub is_subscribed: Option<bool>,

    /// Subscription tier level
    pub subscription_tier: Option<String>,

    /// When subscription expires
    pub subscription_expires_at: Option<DateTime<Utc>>,

    /// ID of the subscription owner
    pub owner_id: Option<Uuid>,

    /// Soft delete flag (true = active, false = deleted)
    pub is_active: Option<bool>,

    /// Current operational status
    pub business_status: Option<String>,

    /// When record was created
    pub created_at: DateTime<Utc>,

    /// When record was last modified
    pub updated_at: DateTime<Utc>,

    /// When data was last verified with external sources
    pub last_verified_at: Option<DateTime<Utc>>,

    /// Primary photo URL derived from associated assets
    #[sqlx(skip)]
    pub primary_photo_url: Option<String>,

    /// Primary thumbnail URL for faster listing rendering
    #[sqlx(skip)]
    pub primary_photo_thumbnail_url: Option<String>,
}

/// Request DTO for creating a new place
/// DOCUMENTATION: Data transfer object for POST /places endpoint
/// Used for API input validation and database inserts
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreatePlaceRequest {
    /// Place name (required)
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Place type (required)
    #[serde(rename = "type")]
    pub type_: String,

    /// Geographic location [longitude, latitude]
    pub location: [f64; 2],

    /// Physical address
    pub address: Option<String>,

    /// City name (required)
    pub city: String,

    /// Neighborhood (optional)
    pub district: Option<String>,

    /// Phone number
    pub phone: Option<String>,

    /// Website URL
    pub website: Option<String>,

    /// Google Places ID (for deduplication)
    pub google_place_id: Option<String>,

    /// Postal code
    #[serde(default)]
    pub postal_code: Option<String>,

    /// Initial categories
    pub main_categories: Vec<String>,

    /// Secondary categories
    #[serde(default)]
    pub secondary_categories: Vec<String>,

    /// Cuisine types
    #[serde(default)]
    pub cuisine_types: Vec<String>,

    /// Google rating
    #[serde(default)]
    pub google_rating: Option<f32>,

    /// Total number of Google ratings
    #[serde(default)]
    pub google_rating_count: Option<i32>,

    /// Price level (0-4)
    #[serde(default)]
    pub price_level: Option<i32>,

    /// Google Maps URL
    #[serde(default)]
    pub google_place_url: Option<String>,

    /// Opening hours metadata
    #[serde(default)]
    pub opening_hours: Option<Value>,

    /// Is currently open
    #[serde(default)]
    pub is_open_now: Option<bool>,

    /// Business status reported by Google
    #[serde(default)]
    pub business_status: Option<String>,

    /// Suitable for tags
    #[serde(default)]
    pub suitable_for: Vec<String>,
}

/// Request DTO for updating an existing place
/// DOCUMENTATION: Data transfer object for PUT /places/{id} endpoint
/// All fields are optional - only provided fields are updated
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePlaceRequest {
    /// Updated name
    pub name: Option<String>,

    /// Updated description
    pub description: Option<String>,

    /// Update custom tags
    pub tags: Option<Value>,

    /// Update vibe descriptor
    pub vibe_descriptor: Option<Value>,

    /// Update operating hours
    pub opening_hours: Option<Value>,

    /// Update Google rating
    pub google_rating: Option<f32>,

    /// Update business status
    pub business_status: Option<String>,
}

/// Response DTO for API responses
/// DOCUMENTATION: Data transfer object for GET endpoints
/// Contains only relevant information for API consumers
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    
    /// Place category (restaurant, bar, cafe, club, museum, park, etc.)
    #[serde(rename = "type")]
    pub type_: String,
    
    /// Geographic coordinates
    pub latitude: f64,
    pub longitude: f64,
    
    /// Full address
    pub address: Option<String>,
    pub city: String,
    pub district: Option<String>,
    pub postal_code: Option<String>,
    
    /// Contact information
    pub phone: Option<String>,
    pub website: Option<String>,
    
    /// Google Places integration
    pub google_place_id: Option<String>,
    pub google_place_url: Option<String>,
    pub google_rating: Option<f32>,
    pub google_rating_count: Option<i32>,
    
    /// Price level: 0 (free) to 4 (very expensive)
    pub price_level: Option<i32>,
    
    /// Classification
    pub main_categories: Option<Vec<String>>,
    pub secondary_categories: Option<Vec<String>>,
    pub cuisine_types: Option<Vec<String>>,
    
    /// Metadata
    pub tags: Option<Value>,
    pub vibe_descriptor: Option<Value>,
    pub suitable_for: Option<Vec<String>>,
    
    /// Operating hours
    pub opening_hours: Option<Value>,
    pub is_open_now: Option<bool>,
    
    /// Business information
    pub business_status: Option<String>,
    pub is_subscribed: Option<bool>,
    
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    /// Media
    pub primary_photo_url: Option<String>,
    pub primary_photo_thumbnail_url: Option<String>,
}

/// Detailed response DTO
/// DOCUMENTATION: Extended response with additional calculated fields
/// Used for GET /places/{id} endpoint
#[derive(Debug, Serialize)]
pub struct PlaceDetailResponse {
    #[serde(flatten)]
    pub place: PlaceResponse,
    pub photos: Vec<PhotoResponse>,
    pub reviews: Vec<ReviewResponse>,
}

/// Search query parameters
/// DOCUMENTATION: DTO for parsing query string in /places/search endpoint
/// All parameters are optional for flexible searching
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Full-text search query
    pub q: Option<String>,

    /// Filter by city
    pub city: Option<String>,

    /// Filter by neighborhood
    pub district: Option<String>,

    /// Filter by place type
    #[serde(rename = "type")]
    pub type_: Option<String>,

    /// Geographic latitude (for proximity search)
    pub lat: Option<f64>,

    /// Geographic longitude (for proximity search)
    pub lon: Option<f64>,

    /// Search radius in kilometers
    pub radius_km: Option<f64>,

    /// Minimum rating filter
    pub min_rating: Option<f32>,

    /// Filter by specific tags
    #[allow(dead_code)]
    pub tags: Option<Vec<String>>,

    /// Page number (1-based)
    pub page: Option<i64>,

    /// Results per page (max 100)
    pub limit: Option<i64>,
}

/// Paginated search response
/// DOCUMENTATION: DTO for returning search results with pagination metadata
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// Array of place results
    pub data: Vec<PlaceResponse>,

    /// Total number of matches (regardless of pagination)
    pub total_count: i64,

    /// Current page number
    pub page: i64,

    /// Results per page
    pub limit: i64,

    /// Whether more results exist on next page
    pub has_more: bool,
}

/// Frontend-compatible place response
/// DOCUMENTATION: Response format expected by the frontend
#[derive(Debug, Clone, Serialize)]
pub struct FrontendPlaceResponse {
    /// Google Place ID (used as identifier)
    pub place_id: String,
    
    /// Place name
    pub name: String,
    
    /// Full formatted address
    pub formatted_address: Option<String>,
    
    /// Short vicinity address
    pub vicinity: Option<String>,
    
    /// Geographic coordinates
    pub latitude: f64,
    pub longitude: f64,
    
    /// Place types array
    pub types: Vec<String>,
    
    /// Rating (0-5)
    pub rating: Option<f32>,
    
    /// Total number of ratings
    pub user_ratings_total: Option<i32>,
    
    /// Price level (0-4)
    pub price_level: Option<i32>,
    
    /// Phone number
    pub phone_number: Option<String>,
    
    /// Website URL
    pub website: Option<String>,
    
    /// Opening hours metadata
    pub opening_hours: Option<Value>,
    
    /// Is currently open
    pub is_open: Option<bool>,
    
    /// Distance in kilometers (if search had coordinates)
    pub distance_km: Option<f64>,
    
    /// Custom attributes for frontend
    pub custom_attributes: FrontendCustomAttributes,
}

/// Custom attributes for frontend
#[derive(Debug, Clone, Serialize)]
pub struct FrontendCustomAttributes {
    /// City name
    pub city: Option<String>,
    
    /// District/neighborhood
    pub district: Option<String>,
    
    /// Primary photo URL
    pub primary_photo_url: Option<String>,
    
    /// Primary photo thumbnail URL
    pub primary_photo_thumbnail_url: Option<String>,
    
    /// Google Place ID
    pub google_place_id: String,
    
    /// Array of photos
    pub photos: Vec<FrontendPhotoResponse>,
    
    /// Array of reviews
    pub reviews: Vec<FrontendReviewResponse>,
}

/// Frontend photo response
#[derive(Debug, Clone, Serialize)]
pub struct FrontendPhotoResponse {
    pub photo_url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub attribution: Option<String>,
}

/// Frontend review response
#[derive(Debug, Clone, Serialize)]
pub struct FrontendReviewResponse {
    pub author: Option<String>,
    pub rating: Option<i32>,
    pub text: Option<String>,
    pub relative_time_description: Option<String>,
}

/// Frontend search response
/// DOCUMENTATION: Response format for /places/search endpoint
#[derive(Debug, Serialize)]
pub struct FrontendSearchResponse {
    /// Array of places
    pub places: Vec<FrontendPlaceResponse>,
    
    /// Total number of results
    pub total: i64,
    
    /// Current page
    pub page: i64,
    
    /// Results per page
    pub per_page: i64,
    
    /// Total pages
    pub total_pages: i64,
}

impl Place {
    /// Convert Place to PlaceResponse for API
    /// DOCUMENTATION: Maps database model to API response DTO
    /// Excludes internal fields like search_vector
    pub fn to_response(&self) -> PlaceResponse {
        PlaceResponse {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            type_: self.type_field.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            address: self.address.clone(),
            city: self.city.clone(),
            district: self.district.clone(),
            postal_code: self.postal_code.clone(),
            phone: self.phone.clone(),
            website: self.website.clone(),
            google_place_id: self.google_place_id.clone(),
            google_place_url: self.google_place_url.clone(),
            google_rating: self.google_rating,
            google_rating_count: self.google_rating_count,
            price_level: self.price_level,
            main_categories: self.main_categories.clone(),
            secondary_categories: self.secondary_categories.clone(),
            cuisine_types: self.cuisine_types.clone(),
            tags: self.tags.clone(),
            vibe_descriptor: self.vibe_descriptor.clone(),
            suitable_for: self.suitable_for.clone(),
            opening_hours: self.opening_hours.clone(),
            is_open_now: self.is_open_now,
            business_status: self.business_status.clone(),
            is_subscribed: self.is_subscribed,
            created_at: self.created_at,
            updated_at: self.updated_at,
            primary_photo_url: self.primary_photo_url.clone(),
            primary_photo_thumbnail_url: self.primary_photo_thumbnail_url.clone(),
        }
    }
}
