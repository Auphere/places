// src/services/google_places_client.rs
// DOCUMENTATION: Google Places API client
// PURPOSE: Handle communication with Google Places API for place data retrieval

use crate::errors::PlacesError;
use crate::models::CreatePlaceRequest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Google Places API client
/// DOCUMENTATION: Handles authentication and API calls to Google Places
pub struct GooglePlacesClient {
    /// HTTP client for making requests
    client: Client,
    /// Google Places API key
    api_key: String,
    /// Base URL for Google Places API
    base_url: String,
}

/// Response from Google Places Nearby Search
/// DOCUMENTATION: Parsed response from Google Places API
#[derive(Debug, Deserialize, Serialize)]
pub struct GooglePlacesResponse {
    /// Results array from API
    pub results: Vec<GooglePlace>,
    /// Status of the API call
    pub status: String,
    /// Next page token (if more results available)
    pub next_page_token: Option<String>,
    /// Error message (if status is not OK)
    pub error_message: Option<String>,
}

/// Individual place from Google Places API
/// DOCUMENTATION: Place data structure returned by Google Places
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GooglePlace {
    /// Google's unique place identifier
    pub place_id: String,
    /// Place name
    pub name: String,
    /// Place types array (e.g., ["restaurant", "food", "point_of_interest"])
    pub types: Vec<String>,
    /// Geographic location
    pub geometry: GoogleGeometry,
    /// Formatted address (detailed, from Place Details)
    pub formatted_address: Option<String>,
    /// Vicinity (short address, from Nearby Search)
    pub vicinity: Option<String>,
    /// Address components (city, district, postal code, etc.)
    pub address_components: Option<Vec<GoogleAddressComponent>>,
    /// Rating (0-5)
    pub rating: Option<f32>,
    /// Number of user ratings
    pub user_ratings_total: Option<i32>,
    /// Price level (0-4: free to very expensive)
    pub price_level: Option<i32>,
    /// Business status (OPERATIONAL, CLOSED_TEMPORARILY, etc.)
    pub business_status: Option<String>,
    /// Opening hours indicator
    pub opening_hours: Option<GoogleOpeningHours>,
    /// Phone number (formatted for local use)
    pub formatted_phone_number: Option<String>,
    /// Phone number (international format)
    pub international_phone_number: Option<String>,
    /// Website URL
    pub website: Option<String>,
    /// Google Maps URL
    pub url: Option<String>,
    /// User reviews (from Place Details)
    pub reviews: Option<Vec<GoogleReview>>,
    /// Photos (from Place Details)
    pub photos: Option<Vec<GooglePhoto>>,
}

/// Geographic location from Google
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleGeometry {
    /// Location coordinates
    pub location: GoogleLocation,
}

/// Coordinates from Google
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleLocation {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lng: f64,
}

/// Address component from Google Places
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleAddressComponent {
    /// Long name (e.g., "Zaragoza", "50001")
    pub long_name: String,
    /// Short name (e.g., "Zaragoza", "50001")
    pub short_name: String,
    /// Types of this component (e.g., ["locality", "political"])
    pub types: Vec<String>,
}

/// Opening hours metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleOpeningHours {
    /// Whether place is currently open
    pub open_now: Option<bool>,
    /// Detailed regular opening hours
    pub weekday_text: Option<Vec<String>>,
    /// Opening periods
    pub periods: Option<Vec<GoogleOpeningPeriod>>,
}

/// Google opening period metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleOpeningPeriod {
    pub open: Option<GoogleOpeningTime>,
    pub close: Option<GoogleOpeningTime>,
}

/// Google opening time entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleOpeningTime {
    pub day: Option<i32>,
    pub time: Option<String>,
}

/// Review from Google Places
/// DOCUMENTATION: User review data structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleReview {
    /// Review author name
    pub author_name: Option<String>,
    /// Rating (1-5)
    pub rating: Option<i32>,
    /// Review text
    pub text: Option<String>,
    /// Time of review (Unix timestamp)
    pub time: Option<i64>,
    /// Relative time description (e.g., "a month ago")
    pub relative_time_description: Option<String>,
    /// Profile photo URL
    pub profile_photo_url: Option<String>,
}

/// Photo from Google Places
/// DOCUMENTATION: Place photo data structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GooglePhoto {
    /// Photo reference (used to fetch actual photo)
    pub photo_reference: String,
    /// Photo width in pixels
    pub width: Option<i32>,
    /// Photo height in pixels
    pub height: Option<i32>,
    /// HTML attributions (required by Google)
    pub html_attributions: Option<Vec<String>>,
}

impl GooglePlacesClient {
    /// Create new Google Places API client
    /// DOCUMENTATION: Initializes client with API key
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://maps.googleapis.com/maps/api/place".to_string(),
        }
    }

    /// Get API key
    /// DOCUMENTATION: Returns the Google Places API key
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    /// Perform nearby search for places
    /// DOCUMENTATION: Searches for places near a geographic point
    ///
    /// # Arguments
    /// * `latitude` - Center point latitude
    /// * `longitude` - Center point longitude
    /// * `radius` - Search radius in meters (max 50000)
    /// * `place_type` - Optional type filter (e.g., "restaurant", "bar")
    /// * `keyword` - Optional keyword search
    ///
    /// # Returns
    /// Vector of GooglePlace results
    pub async fn nearby_search(
        &self,
        latitude: f64,
        longitude: f64,
        radius: u32,
        place_type: Option<&str>,
        keyword: Option<&str>,
    ) -> Result<Vec<GooglePlace>, PlacesError> {
        let url = format!("{}/nearbysearch/json", self.base_url);

        let mut params = HashMap::new();
        params.insert("location", format!("{},{}", latitude, longitude));
        params.insert("radius", radius.to_string());
        params.insert("key", self.api_key.clone());

        if let Some(pt) = place_type {
            params.insert("type", pt.to_string());
        }

        if let Some(kw) = keyword {
            params.insert("keyword", kw.to_string());
        }

        log::debug!(
            "Google Places nearby search: lat={}, lng={}, radius={}",
            latitude,
            longitude,
            radius
        );

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                log::error!("Google Places API request failed: {}", e);
                PlacesError::ExternalApiError(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("Google Places API error {}: {}", status, body);
            return Err(PlacesError::ExternalApiError(format!(
                "API error {}: {}",
                status, body
            )));
        }

        let api_response: GooglePlacesResponse = response.json().await.map_err(|e| {
            log::error!("Failed to parse Google Places response: {}", e);
            PlacesError::ExternalApiError(format!("Parse error: {}", e))
        })?;

        // Check API response status
        match api_response.status.as_str() {
            "OK" | "ZERO_RESULTS" => {
                log::info!(
                    "Google Places search returned {} results",
                    api_response.results.len()
                );
                Ok(api_response.results)
            }
            "OVER_QUERY_LIMIT" => {
                log::error!("Google Places API quota exceeded");
                Err(PlacesError::RateLimitExceeded)
            }
            "REQUEST_DENIED" | "INVALID_REQUEST" => {
                let msg = api_response
                    .error_message
                    .unwrap_or_else(|| "Unknown error".to_string());
                log::error!("Google Places API request denied: {}", msg);
                Err(PlacesError::ExternalApiError(msg))
            }
            other => {
                let msg = api_response
                    .error_message
                    .unwrap_or_else(|| format!("Unknown status: {}", other));
                log::error!("Google Places API unexpected status: {}", msg);
                Err(PlacesError::ExternalApiError(msg))
            }
        }
    }

    /// Get photo URL from photo reference
    /// DOCUMENTATION: Converts Google photo_reference to actual photo URL
    ///
    /// # Arguments
    /// * `photo_reference` - Photo reference from Google Places API
    /// * `max_width` - Maximum width in pixels (default 800)
    ///
    /// # Returns
    /// Photo URL that can be used directly in img tags
    pub fn get_photo_url(&self, photo_reference: &str, max_width: Option<i32>) -> String {
        let width = max_width.unwrap_or(800);
        format!(
            "{}/photo?maxwidth={}&photoreference={}&key={}",
            self.base_url, width, photo_reference, self.api_key
        )
    }

    /// Get thumbnail photo URL from photo reference
    /// DOCUMENTATION: Converts Google photo_reference to thumbnail URL (smaller size)
    ///
    /// # Arguments
    /// * `photo_reference` - Photo reference from Google Places API
    ///
    /// # Returns
    /// Thumbnail photo URL (400px width)
    pub fn get_photo_thumbnail_url(&self, photo_reference: &str) -> String {
        self.get_photo_url(photo_reference, Some(400))
    }

    /// Get detailed information about a specific place
    /// DOCUMENTATION: Retrieves detailed place information by place_id
    ///
    /// # Arguments
    /// * `place_id` - Google Place ID
    ///
    /// # Returns
    /// Detailed place information including photos and reviews
    pub async fn get_place_details(&self, place_id: &str) -> Result<GooglePlace, PlacesError> {
        let url = format!("{}/details/json", self.base_url);

        let params = [
            ("place_id", place_id),
            ("key", &self.api_key),
            // Request comprehensive place information including reviews, photos, and address components
            ("fields", "name,place_id,geometry,formatted_address,address_components,vicinity,rating,user_ratings_total,price_level,types,business_status,opening_hours,formatted_phone_number,international_phone_number,website,url,reviews,photos"),
        ];

        log::debug!("Google Places details lookup: place_id={}", place_id);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                log::error!("Google Places details request failed: {}", e);
                PlacesError::ExternalApiError(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(PlacesError::ExternalApiError(
                "Details request failed".to_string(),
            ));
        }

        #[derive(Deserialize)]
        struct DetailsResponse {
            result: GooglePlace,
            status: String,
        }

        let api_response: DetailsResponse = response
            .json()
            .await
            .map_err(|e| PlacesError::ExternalApiError(format!("Parse error: {}", e)))?;

        if api_response.status == "OK" {
            Ok(api_response.result)
        } else {
            Err(PlacesError::ExternalApiError(format!(
                "Details status: {}",
                api_response.status
            )))
        }
    }

    /// Convert GooglePlace to CreatePlaceRequest
    /// DOCUMENTATION: Maps Google Places API response to internal place creation request
    /// Extracts all available information from Google Places including:
    /// - District and postal code from address_components
    /// - Cuisine types from place types and name
    /// - Suitable_for tags based on characteristics
    /// - Google Maps URL
    /// - Price level
    ///
    /// # Arguments
    /// * `google_place` - Place data from Google API
    /// * `city` - City name to assign to the place
    ///
    /// # Returns
    /// CreatePlaceRequest ready for database insertion
    pub fn to_create_request(&self, google_place: &GooglePlace, city: &str) -> CreatePlaceRequest {
        // Map Google place types to our internal type
        let place_type = self.map_google_type_to_internal(&google_place.types);

        // Extract main categories from Google types (excluding generic ones)
        let main_categories: Vec<String> = google_place
            .types
            .iter()
            .filter(|t| {
                !t.starts_with("point_of_interest") 
                && !t.starts_with("establishment")
                && *t != "geocode"
            })
            .take(3)
            .map(|s| s.clone())
            .collect();

        // Extract secondary categories (remaining types)
        let secondary_categories: Vec<String> = google_place
            .types
            .iter()
            .filter(|t| {
                !t.starts_with("point_of_interest") 
                && !t.starts_with("establishment")
                && *t != "geocode"
            })
            .skip(3)
            .take(5)
            .map(|s| s.clone())
            .collect();

        // Extract cuisine types for restaurants
        let cuisine_types = if place_type == "restaurant" || place_type == "cafe" {
            self.extract_cuisine_types(&google_place.types, &google_place.name)
        } else {
            Vec::new()
        };

        // Extract district and postal code from address components
        let district = self.extract_district(&google_place.address_components);
        let postal_code = self.extract_postal_code(&google_place.address_components);

        // Determine suitable_for tags
        let suitable_for = self.determine_suitable_for(&google_place.types, google_place.price_level);

        // Generate Google Maps URL if available, otherwise construct from place_id
        let google_place_url = google_place.url.clone().or_else(|| {
            Some(format!(
                "https://www.google.com/maps/place/?q=place_id:{}",
                google_place.place_id
            ))
        });

        // Extract is_open_now if available
        let is_open_now = google_place
            .opening_hours
            .as_ref()
            .and_then(|hours| hours.open_now);

        CreatePlaceRequest {
            name: google_place.name.clone(),
            // Don't duplicate address in description - leave it None for now
            // Can be enriched later with LLM-generated descriptions
            description: None,
            type_: place_type,
            location: [
                google_place.geometry.location.lng,
                google_place.geometry.location.lat,
            ],
            // Prefer formatted_address over vicinity (more detailed)
            address: google_place
                .formatted_address
                .clone()
                .or_else(|| google_place.vicinity.clone()),
            city: city.to_string(),
            district,
            postal_code,
            // Use phone number from Place Details (if available)
            phone: google_place
                .formatted_phone_number
                .clone()
                .or_else(|| google_place.international_phone_number.clone()),
            // Use website from Place Details (if available)
            website: google_place.website.clone(),
            google_place_id: Some(google_place.place_id.clone()),
            main_categories,
            secondary_categories,
            cuisine_types,
            google_rating: google_place.rating,
            google_rating_count: google_place.user_ratings_total,
            price_level: google_place.price_level,
            google_place_url,
            opening_hours: google_place
                .opening_hours
                .as_ref()
                .and_then(|hours| serde_json::to_value(hours).ok()),
            is_open_now,
            business_status: google_place.business_status.clone(),
            suitable_for,
        }
    }

    /// Map Google place types to internal place type
    /// DOCUMENTATION: Converts Google's type array to our single type field
    /// Priority order: restaurant > bar > cafe > club > museum > park > other
    fn map_google_type_to_internal(&self, types: &[String]) -> String {
        // Define priority mapping
        let type_map: Vec<(&str, &str)> = vec![
            ("restaurant", "restaurant"),
            ("bar", "bar"),
            ("night_club", "nightclub"),
            ("nightclub", "nightclub"),
            ("cafe", "cafe"),
            ("museum", "other"),
            ("park", "other"),
            ("shopping_mall", "other"),
            ("lodging", "other"),
            ("food", "restaurant"),
            ("meal_takeaway", "restaurant"),
            ("meal_delivery", "restaurant"),
        ];

        // Find first matching type
        for (google_type, internal_type) in type_map {
            if types.iter().any(|t| t == google_type) {
                return internal_type.to_string();
            }
        }

        // Default to "other" if no match
        "other".to_string()
    }

    /// Extract district/neighborhood from address components
    /// DOCUMENTATION: Looks for sublocality, neighborhood, or administrative_area_level_3
    fn extract_district(&self, address_components: &Option<Vec<GoogleAddressComponent>>) -> Option<String> {
        address_components.as_ref().and_then(|components| {
            for component in components {
                if component.types.iter().any(|t| {
                    t == "sublocality" 
                    || t == "sublocality_level_1"
                    || t == "neighborhood"
                    || t == "administrative_area_level_3"
                }) {
                    return Some(component.long_name.clone());
                }
            }
            None
        })
    }

    /// Extract postal code from address components
    /// DOCUMENTATION: Looks for postal_code type
    fn extract_postal_code(&self, address_components: &Option<Vec<GoogleAddressComponent>>) -> Option<String> {
        address_components.as_ref().and_then(|components| {
            for component in components {
                if component.types.contains(&"postal_code".to_string()) {
                    return Some(component.long_name.clone());
                }
            }
            None
        })
    }

    /// Extract cuisine types from Google place types and name
    /// DOCUMENTATION: Maps Google types to cuisine categories
    fn extract_cuisine_types(&self, types: &[String], name: &str) -> Vec<String> {
        let mut cuisines = Vec::new();
        let name_lower = name.to_lowercase();

        // Mapping from Google types to cuisine types
        let cuisine_map: Vec<(&str, &str)> = vec![
            ("italian_restaurant", "italian"),
            ("chinese_restaurant", "chinese"),
            ("japanese_restaurant", "japanese"),
            ("mexican_restaurant", "mexican"),
            ("indian_restaurant", "indian"),
            ("spanish_restaurant", "spanish"),
            ("french_restaurant", "french"),
            ("thai_restaurant", "thai"),
            ("american_restaurant", "american"),
            ("mediterranean_restaurant", "mediterranean"),
        ];

        for (google_type, cuisine) in cuisine_map {
            if types.iter().any(|t| t == google_type) {
                cuisines.push(cuisine.to_string());
            }
        }

        // Check name for cuisine keywords
        let name_keywords = vec![
            ("italian", "italian"),
            ("pizza", "italian"),
            ("sushi", "japanese"),
            ("ramen", "japanese"),
            ("taco", "mexican"),
            ("burrito", "mexican"),
            ("curry", "indian"),
            ("tapas", "spanish"),
            ("paella", "spanish"),
            ("burger", "american"),
            ("bbq", "american"),
            ("thai", "thai"),
            ("vietnamese", "vietnamese"),
            ("korean", "korean"),
            ("mediterranean", "mediterranean"),
            ("chinese", "chinese"),
        ];

        for (keyword, cuisine) in name_keywords {
            if name_lower.contains(keyword) && !cuisines.contains(&cuisine.to_string()) {
                cuisines.push(cuisine.to_string());
            }
        }

        cuisines
    }

    /// Determine suitable_for tags based on place characteristics
    /// DOCUMENTATION: Derives suitable_for tags from types and price level
    fn determine_suitable_for(&self, types: &[String], price_level: Option<i32>) -> Vec<String> {
        let mut suitable = Vec::new();

        // Everyone can go to parks, museums, etc.
        if types.iter().any(|t| t == "park" || t == "museum" || t == "tourist_attraction") {
            suitable.push("families".to_string());
            suitable.push("solo".to_string());
            suitable.push("groups".to_string());
        }

        // Bars and nightclubs
        if types.iter().any(|t| t == "bar" || t == "night_club" || t == "nightclub") {
            suitable.push("groups".to_string());
            suitable.push("couples".to_string());
        }

        // Cafes
        if types.iter().any(|t| t == "cafe") {
            suitable.push("solo".to_string());
            suitable.push("couples".to_string());
            suitable.push("groups".to_string());
        }

        // Restaurants
        if types.iter().any(|t| t == "restaurant" || t == "food") {
            suitable.push("couples".to_string());
            suitable.push("families".to_string());
            suitable.push("groups".to_string());
            
            // Expensive restaurants more suitable for couples
            if let Some(level) = price_level {
                if level >= 3 {
                    // Remove families for very expensive places
                    suitable.retain(|s| s != "families");
                } else if level <= 1 {
                    // Cheap places good for students
                    suitable.push("budget".to_string());
                }
            }
        }

        suitable.dedup();
        suitable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mapping() {
        let client = GooglePlacesClient::new("test_key".to_string());

        let types1 = vec!["restaurant".to_string(), "food".to_string()];
        assert_eq!(client.map_google_type_to_internal(&types1), "restaurant");

        let types2 = vec!["bar".to_string(), "night_club".to_string()];
        assert_eq!(client.map_google_type_to_internal(&types2), "bar");

        let types3 = vec!["store".to_string(), "shop".to_string()];
        assert_eq!(client.map_google_type_to_internal(&types3), "other");
    }

    #[test]
    fn test_to_create_request() {
        let client = GooglePlacesClient::new("test_key".to_string());

        let google_place = GooglePlace {
            place_id: "ChIJ123".to_string(),
            name: "Test Restaurant".to_string(),
            types: vec!["restaurant".to_string(), "food".to_string()],
            geometry: GoogleGeometry {
                location: GoogleLocation {
                    lat: 40.4168,
                    lng: -3.7038,
                },
            },
            formatted_address: Some("Calle Mayor 1, 28013 Madrid, Spain".to_string()),
            vicinity: Some("Calle Mayor, Madrid".to_string()),
            address_components: Some(vec![
                GoogleAddressComponent {
                    long_name: "28013".to_string(),
                    short_name: "28013".to_string(),
                    types: vec!["postal_code".to_string()],
                },
                GoogleAddressComponent {
                    long_name: "Centro".to_string(),
                    short_name: "Centro".to_string(),
                    types: vec!["sublocality".to_string(), "political".to_string()],
                },
            ]),
            rating: Some(4.5),
            user_ratings_total: Some(100),
            price_level: Some(2),
            business_status: Some("OPERATIONAL".to_string()),
            opening_hours: Some(GoogleOpeningHours {
                open_now: Some(true),
                weekday_text: Some(vec!["Monday: 09:00 – 22:00".to_string()]),
                periods: None,
            }),
            formatted_phone_number: Some("+34 912 345 678".to_string()),
            international_phone_number: Some("+34 912 345 678".to_string()),
            website: Some("https://testrestaurant.com".to_string()),
            url: Some("https://maps.google.com/?cid=123".to_string()),
            reviews: None,
            photos: None,
        };

        let request = client.to_create_request(&google_place, "Madrid");

        assert_eq!(request.name, "Test Restaurant");
        assert_eq!(request.city, "Madrid");
        assert_eq!(request.type_, "restaurant");
        assert_eq!(request.location, [-3.7038, 40.4168]);
        assert_eq!(request.google_place_id, Some("ChIJ123".to_string()));
        assert_eq!(request.google_rating, Some(4.5));
        assert_eq!(request.google_rating_count, Some(100));
        assert_eq!(request.price_level, Some(2));
        assert_eq!(request.district, Some("Centro".to_string()));
        assert_eq!(request.postal_code, Some("28013".to_string()));
        assert_eq!(request.business_status, Some("OPERATIONAL".to_string()));
        assert!(request.opening_hours.is_some());
        assert_eq!(request.is_open_now, Some(true));
        assert!(request.description.is_none()); // Should not duplicate address
    }
}
