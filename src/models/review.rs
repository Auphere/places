// src/models/review.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Place review from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub place_id: Uuid,
    pub source: String,
    pub source_id: Option<String>,
    pub author: Option<String>,
    pub rating: f32,
    pub text: Option<String>,
    pub posted_at: DateTime<Utc>,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<f32>,
    pub extracted_tags: Option<serde_json::Value>,
    pub helpful_count: Option<i32>,
    pub response_from_owner: Option<String>,
    pub is_verified: Option<bool>,
    pub has_photo: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewRequest {
    pub place_id: Uuid,
    pub source: String,
    pub source_id: Option<String>,
    pub author: Option<String>,
    pub rating: f32,
    pub text: Option<String>,
    pub posted_at: DateTime<Utc>,
    pub is_verified: Option<bool>,
    pub has_photo: Option<bool>,
}

/// Review response DTO exposed via API
#[derive(Debug, Clone, Serialize)]
pub struct ReviewResponse {
    pub id: Uuid,
    pub source: String,
    pub author: Option<String>,
    pub rating: f32,
    pub text: Option<String>,
    pub posted_at: DateTime<Utc>,
    pub helpful_count: Option<i32>,
    pub is_verified: Option<bool>,
}

impl Review {
    /// Convert database Review into API response
    pub fn to_response(&self) -> ReviewResponse {
        ReviewResponse {
            id: self.id,
            source: self.source.clone(),
            author: self.author.clone(),
            rating: self.rating,
            text: self.text.clone(),
            posted_at: self.posted_at,
            helpful_count: self.helpful_count,
            is_verified: self.is_verified,
        }
    }
}
