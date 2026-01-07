// src/services/foursquare_client.rs
// DOCUMENTATION: Foursquare Places API client (v3)
// PURPOSE: Fetch enrichment (match/search + photos) from Foursquare with caching.

use crate::errors::PlacesError;
use crate::services::PlacesCache;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Foursquare Places API base URL (per official docs)
const FOURSQUARE_BASE_URL: &str = "https://places-api.foursquare.com";
// Required header per Foursquare Places API docs
const FOURSQUARE_API_VERSION: &str = "2025-06-17";

pub struct FoursquareClient {
    client: Client,
    api_key: String,
    cache: Arc<PlacesCache>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqSearchResponse {
    pub results: Vec<FsqPlaceResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqPlaceResult {
    pub fsq_id: String,
    pub name: String,
    pub distance: Option<i32>,
    pub categories: Option<Vec<FsqCategory>>,
    pub location: Option<FsqLocation>,
    pub geocodes: Option<FsqGeocodes>,
    pub rating: Option<f32>, // 0-10, may be missing depending on access
    pub popularity: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqLocation {
    pub formatted_address: Option<String>,
    pub locality: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub neighborhood: Option<Vec<String>>,
    pub address: Option<String>,
    pub postcode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqGeocodes {
    pub main: Option<FsqLatLng>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqLatLng {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqPhoto {
    pub id: String,
    pub prefix: String,
    pub suffix: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsqTip {
    pub id: String,
    pub created_at: Option<String>,
    pub text: Option<String>,
    pub agree_count: Option<i32>,
    pub disagree_count: Option<i32>,
}

impl FoursquareClient {
    fn build_http_client() -> Client {
        Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|e| {
                log::warn!("Failed to build reqwest client with no_proxy: {}", e);
                Client::builder()
                    .build()
                    .expect("Failed to build reqwest client")
            })
    }

    pub fn new(api_key: String, cache: Arc<PlacesCache>) -> Self {
        Self {
            client: Self::build_http_client(),
            api_key,
            cache,
        }
    }

    fn with_required_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .header("X-Places-Api-Version", FOURSQUARE_API_VERSION)
    }

    /// Find the best Foursquare match near a lat/lon for a given name.
    /// DOCUMENTATION: Lightweight matching call used to attach fsq_id to a Google place.
    pub async fn match_place(
        &self,
        name: &str,
        lat: f64,
        lon: f64,
        radius_m: u32,
    ) -> Result<Option<FsqPlaceResult>, PlacesError> {
        let name_trim = name.trim();
        if name_trim.is_empty() {
            return Ok(None);
        }

        let cache_key = format!(
            "fsq:match:{}:{}:{}:{}",
            name_trim.to_lowercase(),
            (lat * 1000.0).round() as i64,
            (lon * 1000.0).round() as i64,
            radius_m
        );

        if let Some(cached_json) = self.cache.get(&cache_key).await {
            if let Ok(hit) = serde_json::from_str::<Option<FsqPlaceResult>>(&cached_json) {
                return Ok(hit);
            }
        }

        let url = format!("{}/places/search", FOURSQUARE_BASE_URL);
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("query", name_trim.to_string());
        params.insert("ll", format!("{},{}", lat, lon));
        params.insert("radius", radius_m.to_string());
        params.insert("limit", "1".to_string());

        // Prefer fields we can use for matching/metadata; keep it minimal.
        params.insert(
            "fields",
            "fsq_id,name,distance,categories,location,geocodes,rating,popularity".to_string(),
        );

        let resp = self
            .with_required_headers(self.client.get(url))
            .query(&params)
            .send()
            .await
            .map_err(|e| PlacesError::ExternalApiError(format!("Foursquare request failed: {}", e)))?;

        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(PlacesError::RateLimitExceeded);
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PlacesError::ExternalApiError(format!(
                "Foursquare error {}: {}",
                status, body
            )));
        }

        let parsed: FsqSearchResponse = resp.json().await.map_err(|e| {
            PlacesError::ExternalApiError(format!("Foursquare parse error: {}", e))
        })?;

        let best = parsed.results.into_iter().next();
        if let Ok(json) = serde_json::to_string(&best) {
            self.cache.set(cache_key, json).await;
        }
        Ok(best)
    }

    /// Fetch photos for a Foursquare place.
    pub async fn get_photos(&self, fsq_id: &str, limit: u32) -> Result<Vec<FsqPhoto>, PlacesError> {
        let id = fsq_id.trim();
        if id.is_empty() {
            return Ok(Vec::new());
        }

        let cache_key = format!("fsq:photos:{}:{}", id, limit);
        if let Some(cached_json) = self.cache.get(&cache_key).await {
            if let Ok(hit) = serde_json::from_str::<Vec<FsqPhoto>>(&cached_json) {
                return Ok(hit);
            }
        }

        let url = format!("{}/places/{}/photos", FOURSQUARE_BASE_URL, id);
        let resp = self
            .with_required_headers(self.client.get(url))
            .query(&[("limit", limit.to_string())])
            .send()
            .await
            .map_err(|e| PlacesError::ExternalApiError(format!("Foursquare request failed: {}", e)))?;

        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(PlacesError::RateLimitExceeded);
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PlacesError::ExternalApiError(format!(
                "Foursquare error {}: {}",
                status, body
            )));
        }

        let parsed: Vec<FsqPhoto> = resp.json().await.map_err(|e| {
            PlacesError::ExternalApiError(format!("Foursquare parse error: {}", e))
        })?;

        if let Ok(json) = serde_json::to_string(&parsed) {
            self.cache.set(cache_key, json).await;
        }
        Ok(parsed)
    }

    /// Fetch tips for a Foursquare place.
    pub async fn get_tips(&self, fsq_id: &str, limit: u32) -> Result<Vec<FsqTip>, PlacesError> {
        let id = fsq_id.trim();
        if id.is_empty() {
            return Ok(Vec::new());
        }

        let cache_key = format!("fsq:tips:{}:{}", id, limit);
        if let Some(cached_json) = self.cache.get(&cache_key).await {
            if let Ok(hit) = serde_json::from_str::<Vec<FsqTip>>(&cached_json) {
                return Ok(hit);
            }
        }

        let url = format!("{}/places/{}/tips", FOURSQUARE_BASE_URL, id);
        let resp = self
            .with_required_headers(self.client.get(url))
            .query(&[("limit", limit.to_string())])
            .send()
            .await
            .map_err(|e| PlacesError::ExternalApiError(format!("Foursquare request failed: {}", e)))?;

        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(PlacesError::RateLimitExceeded);
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PlacesError::ExternalApiError(format!(
                "Foursquare error {}: {}",
                status, body
            )));
        }

        let parsed: Vec<FsqTip> = resp.json().await.map_err(|e| {
            PlacesError::ExternalApiError(format!("Foursquare parse error: {}", e))
        })?;

        if let Ok(json) = serde_json::to_string(&parsed) {
            self.cache.set(cache_key, json).await;
        }
        Ok(parsed)
    }

    pub fn photo_url(prefix: &str, size: &str, suffix: &str) -> String {
        format!("{}{}{}", prefix, size, suffix)
    }
}


