// src/models/tip.rs
// DOCUMENTATION: Tip models (no rating) for sources like Foursquare

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tip {
    pub id: Uuid,
    pub place_id: Uuid,
    pub source: String,
    pub source_id: Option<String>,
    pub author: Option<String>,
    pub text: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub like_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTipRequest {
    pub place_id: Uuid,
    pub source: String,
    pub source_id: Option<String>,
    pub author: Option<String>,
    pub text: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub like_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TipResponse {
    pub id: Uuid,
    pub source: String,
    pub author: Option<String>,
    pub text: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub like_count: Option<i32>,
}

impl Tip {
    pub fn to_response(&self) -> TipResponse {
        TipResponse {
            id: self.id,
            source: self.source.clone(),
            author: self.author.clone(),
            text: self.text.clone(),
            posted_at: self.posted_at,
            like_count: self.like_count,
        }
    }
}


