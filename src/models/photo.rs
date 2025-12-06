// src/models/photo.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Place photo from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Photo {
    pub id: Uuid,
    pub place_id: Uuid,
    pub source: String,
    pub source_photo_reference: Option<String>,
    pub photo_url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub attribution: Option<String>,
    pub is_primary: Option<bool>,
    pub display_order: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new photo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePhotoRequest {
    pub place_id: Uuid,
    pub source: String,
    pub source_photo_reference: Option<String>,
    pub photo_url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub attribution: Option<String>,
    pub is_primary: Option<bool>,
    pub display_order: Option<i32>,
}

/// Photo DTO for API responses
#[derive(Debug, Clone, Serialize)]
pub struct PhotoResponse {
    pub id: Uuid,
    pub source: String,
    pub photo_url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub attribution: Option<String>,
    pub is_primary: Option<bool>,
    pub display_order: Option<i32>,
}

impl Photo {
    /// Convert database photo into API response DTO
    pub fn to_response(&self) -> PhotoResponse {
        PhotoResponse {
            id: self.id,
            source: self.source.clone(),
            photo_url: self.photo_url.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
            width: self.width,
            height: self.height,
            attribution: self.attribution.clone(),
            is_primary: self.is_primary,
            display_order: self.display_order,
        }
    }
}
