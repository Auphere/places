// src/db/repository.rs
// DOCUMENTATION: Database access layer - all SQL queries
// PURPOSE: Abstract database operations from business logic

use crate::errors::PlacesError;
use crate::models::*;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Internal struct for mapping database rows to Place struct
/// DOCUMENTATION: Handles PostGIS POINT extraction via ST_X() and ST_Y()
#[derive(Debug, FromRow)]
struct PlaceRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    pub type_field: String,
    pub longitude: f64, // From ST_X(location)
    pub latitude: f64,  // From ST_Y(location)
    pub address: Option<String>,
    pub city: String,
    pub district: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub google_place_id: Option<String>,
    pub google_place_url: Option<String>,
    pub google_rating: Option<f32>,
    pub google_rating_count: Option<i32>,
    pub price_level: Option<i32>,
    pub main_categories: Option<Vec<String>>,
    pub secondary_categories: Option<Vec<String>>,
    pub cuisine_types: Option<Vec<String>>,
    pub tags: Option<Value>,
    pub vibe_descriptor: Option<Value>,
    pub suitable_for: Option<Vec<String>>,
    pub opening_hours: Option<Value>,
    pub is_open_now: Option<bool>,
    pub is_subscribed: Option<bool>,
    pub subscription_tier: Option<String>,
    pub subscription_expires_at: Option<DateTime<Utc>>,
    pub owner_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub business_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub primary_photo_url: Option<String>,
    #[sqlx(default)]
    pub primary_photo_thumbnail_url: Option<String>,
}

impl PlaceRow {
    /// Convert PlaceRow to Place model
    fn to_place(self) -> Place {
        Place {
            id: self.id,
            name: self.name,
            description: self.description,
            type_field: self.type_field,
            longitude: self.longitude,
            latitude: self.latitude,
            address: self.address,
            city: self.city,
            district: self.district,
            postal_code: self.postal_code,
            phone: self.phone,
            email: self.email,
            website: self.website,
            google_place_id: self.google_place_id,
            google_place_url: self.google_place_url,
            google_rating: self.google_rating,
            google_rating_count: self.google_rating_count,
            price_level: self.price_level,
            main_categories: self.main_categories,
            secondary_categories: self.secondary_categories,
            cuisine_types: self.cuisine_types,
            tags: self.tags,
            vibe_descriptor: self.vibe_descriptor,
            suitable_for: self.suitable_for,
            opening_hours: self.opening_hours,
            is_open_now: self.is_open_now,
            is_subscribed: self.is_subscribed,
            subscription_tier: self.subscription_tier,
            subscription_expires_at: self.subscription_expires_at,
            owner_id: self.owner_id,
            is_active: self.is_active,
            business_status: self.business_status,
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_verified_at: self.last_verified_at,
            primary_photo_url: self.primary_photo_url,
            primary_photo_thumbnail_url: self.primary_photo_thumbnail_url,
        }
    }
}

/// PlaceRepository: All database operations for places
/// DOCUMENTATION: Uses query_as for type-safe SQL queries with PostGIS support
pub struct PlaceRepository;

impl PlaceRepository {
    /// Create new place in database
    /// DOCUMENTATION: Inserts place and returns created record
    /// Used by POST /places endpoint
    pub async fn create_place(
        pool: &PgPool,
        req: &CreatePlaceRequest,
    ) -> Result<Place, PlacesError> {
        let inserted: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO places (
                name, description, type, location, address,
                city, district, postal_code, phone, website, 
                google_place_id, google_place_url, google_rating, google_rating_count, price_level,
                main_categories, secondary_categories, cuisine_types,
                opening_hours, is_open_now, business_status, suitable_for,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, 
                ST_SetSRID(ST_MakePoint($4, $5), 4326),
                $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16,
                $17, $18, $19,
                $20, $21, $22, $23,
                NOW(), NOW()
            )
            RETURNING id
            "#,
        )
        .bind(&req.name) // $1
        .bind(&req.description) // $2
        .bind(&req.type_) // $3
        .bind(req.location[0]) // $4 - longitude
        .bind(req.location[1]) // $5 - latitude
        .bind(&req.address) // $6
        .bind(&req.city) // $7
        .bind(&req.district) // $8
        .bind(&req.postal_code) // $9
        .bind(&req.phone) // $10
        .bind(&req.website) // $11
        .bind(&req.google_place_id) // $12
        .bind(&req.google_place_url) // $13
        .bind(&req.google_rating) // $14
        .bind(&req.google_rating_count) // $15
        .bind(&req.price_level) // $16
        .bind(&req.main_categories) // $17
        .bind(&req.secondary_categories) // $18
        .bind(&req.cuisine_types) // $19
        .bind(&req.opening_hours) // $20
        .bind(&req.is_open_now) // $21
        .bind(&req.business_status) // $22
        .bind(&req.suitable_for) // $23
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to create place: {}", e);
            PlacesError::DatabaseError(e.to_string())
        })?;

        let place = Self::get_by_id(pool, inserted.0).await?;
        log::info!("Created place with id: {}", place.id);
        Ok(place)
    }

    /// Upsert a place identified by Google Place ID
    /// Inserts new rows or updates existing ones with latest Google metadata
    pub async fn upsert_google_place(
        pool: &PgPool,
        req: &CreatePlaceRequest,
    ) -> Result<(Place, bool), PlacesError> {
        let google_id = req.google_place_id.as_ref().ok_or_else(|| {
            PlacesError::InvalidInput("google_place_id is required for upsert".into())
        })?;

        // Try insert first - on conflict do nothing so we can detect creation
        let insert_sql = r#"
            INSERT INTO places (
                name, description, type, location, address,
                city, district, postal_code, phone, website, 
                google_place_id, google_place_url, google_rating, google_rating_count, price_level,
                main_categories, secondary_categories, cuisine_types,
                opening_hours, is_open_now, business_status, suitable_for,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3,
                ST_SetSRID(ST_MakePoint($4, $5), 4326),
                $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16,
                $17, $18, $19,
                $20, $21, $22, $23,
                NOW(), NOW()
            )
            ON CONFLICT (google_place_id) DO NOTHING
            RETURNING id
        "#;

        let inserted = sqlx::query_as::<_, (Uuid,)>(insert_sql)
            .bind(&req.name)
            .bind(&req.description)
            .bind(&req.type_)
            .bind(req.location[0])
            .bind(req.location[1])
            .bind(&req.address)
            .bind(&req.city)
            .bind(&req.district)
            .bind(&req.postal_code)
            .bind(&req.phone)
            .bind(&req.website)
            .bind(&req.google_place_id)
            .bind(&req.google_place_url)
            .bind(&req.google_rating)
            .bind(&req.google_rating_count)
            .bind(&req.price_level)
            .bind(&req.main_categories)
            .bind(&req.secondary_categories)
            .bind(&req.cuisine_types)
            .bind(&req.opening_hours)
            .bind(&req.is_open_now)
            .bind(&req.business_status)
            .bind(&req.suitable_for)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                log::error!("Failed to upsert place {}: {}", google_id, e);
                PlacesError::DatabaseError(e.to_string())
            })?;

        if let Some((id,)) = inserted {
            let place = Self::get_by_id(pool, id).await?;
            return Ok((place, true));
        }

        // Update existing record
        let updated_sql = r#"
            UPDATE places
            SET name = $1,
                description = $2,
                type = $3,
                location = ST_SetSRID(ST_MakePoint($4, $5), 4326),
                address = $6,
                city = $7,
                district = $8,
                postal_code = $9,
                phone = $10,
                website = $11,
                google_place_url = $12,
                google_rating = $13,
                google_rating_count = $14,
                price_level = $15,
                main_categories = $16,
                secondary_categories = $17,
                cuisine_types = $18,
                opening_hours = $19,
                is_open_now = $20,
                business_status = $21,
                suitable_for = $22,
                is_active = true,
                updated_at = NOW()
            WHERE google_place_id = $23
            RETURNING id
        "#;

        let updated = sqlx::query_as::<_, (Uuid,)>(updated_sql)
            .bind(&req.name)
            .bind(&req.description)
            .bind(&req.type_)
            .bind(req.location[0])
            .bind(req.location[1])
            .bind(&req.address)
            .bind(&req.city)
            .bind(&req.district)
            .bind(&req.postal_code)
            .bind(&req.phone)
            .bind(&req.website)
            .bind(&req.google_place_url)
            .bind(&req.google_rating)
            .bind(&req.google_rating_count)
            .bind(&req.price_level)
            .bind(&req.main_categories)
            .bind(&req.secondary_categories)
            .bind(&req.cuisine_types)
            .bind(&req.opening_hours)
            .bind(&req.is_open_now)
            .bind(&req.business_status)
            .bind(&req.suitable_for)
            .bind(google_id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                log::error!("Failed to update place {}: {}", google_id, e);
                PlacesError::DatabaseError(e.to_string())
            })?;

        let place = Self::get_by_id(pool, updated.0).await?;
        Ok((place, false))
    }

    /// Retrieve place by Google Place ID
    /// DOCUMENTATION: Used for GET /places/{id} when id is a Google Place ID
    pub async fn get_by_google_place_id(
        pool: &PgPool,
        google_place_id: &str,
    ) -> Result<Place, PlacesError> {
        let row = sqlx::query_as::<_, PlaceRow>(
            r#"
            SELECT 
                p.id, p.name, p.description, p.type,
                ST_X(p.location) as longitude, ST_Y(p.location) as latitude,
                p.address, p.city, p.district, p.postal_code,
                p.phone, p.email, p.website, 
                p.google_place_id, p.google_place_url,
                p.google_rating, p.google_rating_count, p.price_level,
                p.main_categories, p.secondary_categories, p.cuisine_types,
                p.tags, p.vibe_descriptor, p.suitable_for,
                p.opening_hours, p.is_open_now,
                p.is_subscribed, p.subscription_tier, p.subscription_expires_at, p.owner_id,
                p.is_active, p.business_status,
                p.created_at, p.updated_at, p.last_verified_at,
                photo.photo_url as primary_photo_url,
                photo.thumbnail_url as primary_photo_thumbnail_url
            FROM places p
            LEFT JOIN LATERAL (
                SELECT photo_url, thumbnail_url
                FROM place_photos
                WHERE place_id = p.id
                ORDER BY is_primary DESC, display_order ASC, created_at ASC
                LIMIT 1
            ) photo ON true
            WHERE p.google_place_id = $1 AND p.is_active = true
            "#,
        )
        .bind(google_place_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to get place by google_place_id {}: {}", google_place_id, e);
            PlacesError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            log::warn!("Place not found with google_place_id: {}", google_place_id);
            PlacesError::NotFound(format!("Place with google_place_id '{}' not found", google_place_id))
        })?;

        Ok(row.to_place())
    }

    /// Retrieve place by ID
    /// DOCUMENTATION: Used for GET /places/{id} endpoint
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Place, PlacesError> {
        let row = sqlx::query_as::<_, PlaceRow>(
            r#"
            SELECT 
                p.id, p.name, p.description, p.type,
                ST_X(p.location) as longitude, ST_Y(p.location) as latitude,
                p.address, p.city, p.district, p.postal_code,
                p.phone, p.email, p.website, 
                p.google_place_id, p.google_place_url,
                p.google_rating, p.google_rating_count, p.price_level,
                p.main_categories, p.secondary_categories, p.cuisine_types,
                p.tags, p.vibe_descriptor, p.suitable_for,
                p.opening_hours, p.is_open_now,
                p.is_subscribed, p.subscription_tier, p.subscription_expires_at, p.owner_id,
                p.is_active, p.business_status,
                p.created_at, p.updated_at, p.last_verified_at,
                photo.photo_url as primary_photo_url,
                photo.thumbnail_url as primary_photo_thumbnail_url
            FROM places p
            LEFT JOIN LATERAL (
                SELECT photo_url, thumbnail_url
                FROM place_photos
                WHERE place_id = p.id
                ORDER BY is_primary DESC, display_order ASC, created_at ASC
                LIMIT 1
            ) photo ON true
            WHERE p.id = $1 AND p.is_active = true
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            log::error!("Database error fetching place: {}", e);
            PlacesError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            log::warn!("Place not found: {}", id);
            PlacesError::NotFound(id.to_string())
        })?;

        Ok(row.to_place())
    }

    /// Search places with full-text and filters
    /// DOCUMENTATION: Used for GET /places/search endpoint
    /// Returns tuple: (results, total_count) for pagination
    pub async fn search(
        pool: &PgPool,
        query: &SearchQuery,
    ) -> Result<(Vec<Place>, i64), PlacesError> {
        let limit = query.limit.unwrap_or(20).min(100);
        let page = query.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        // Build base SELECT with PostGIS coordinate extraction
        let select_clause = r#"
            SELECT 
                p.id, p.name, p.description, p.type,
                ST_X(p.location) as longitude, ST_Y(p.location) as latitude,
                p.address, p.city, p.district, p.postal_code,
                p.phone, p.email, p.website, 
                p.google_place_id, p.google_place_url,
                p.google_rating, p.google_rating_count, p.price_level,
                p.main_categories, p.secondary_categories, p.cuisine_types,
                p.tags, p.vibe_descriptor, p.suitable_for,
                p.opening_hours, p.is_open_now,
                p.is_subscribed, p.subscription_tier, p.subscription_expires_at, p.owner_id,
                p.is_active, p.business_status,
                p.created_at, p.updated_at, p.last_verified_at,
                photo.photo_url as primary_photo_url,
                photo.thumbnail_url as primary_photo_thumbnail_url
            FROM places p
            LEFT JOIN LATERAL (
                SELECT photo_url, thumbnail_url
                FROM place_photos
                WHERE place_id = p.id
                ORDER BY is_primary DESC, display_order ASC, created_at ASC
                LIMIT 1
            ) photo ON true
        "#;

        // Build dynamic query based on provided filters
        let mut where_clauses = vec!["p.is_active = true".to_string()];

        // Full-text search
        if let Some(q) = &query.q {
            where_clauses.push(format!(
                "p.search_vector @@ plainto_tsquery('english', '{}')",
                q.replace("'", "''")
            ));
        }

        // City filter
        if let Some(city) = &query.city {
            where_clauses.push(format!("p.city ILIKE '%{}%'", city.replace("'", "''")));
        }

        // District filter
        if let Some(district) = &query.district {
            where_clauses.push(format!(
                "p.district ILIKE '%{}%'",
                district.replace("'", "''")
            ));
        }

        // Type filter
        if let Some(type_) = &query.type_ {
            where_clauses.push(format!("p.type = '{}'", type_.replace("'", "''")));
        }

        // Geographic proximity
        if let (Some(lat), Some(lon), Some(radius_km)) = (query.lat, query.lon, query.radius_km) {
            where_clauses.push(format!(
                "ST_DWithin(p.location::geography, ST_SetSRID(ST_MakePoint({}, {}), 4326)::geography, {})",
                lon, lat, radius_km * 1000.0
            ));
        }

        // Rating filter
        if let Some(min_rating) = query.min_rating {
            where_clauses.push(format!("p.google_rating >= {}", min_rating));
        }

        let where_clause = format!("WHERE {}", where_clauses.join(" AND "));

        // Get total count
        let count_sql = format!("SELECT COUNT(*) FROM places p {}", where_clause);
        let count_result: (i64,) =
            sqlx::query_as(&count_sql)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    log::error!("Count query error: {}", e);
                    PlacesError::DatabaseError(e.to_string())
                })?;

        let total = count_result.0;

        // Build final query with ordering and pagination
        let sql = format!(
            "{} {} ORDER BY p.google_rating DESC NULLS LAST LIMIT {} OFFSET {}",
            select_clause, where_clause, limit, offset
        );

        log::debug!("Executing search query: {}", sql);

        let rows = sqlx::query_as::<_, PlaceRow>(&sql)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                log::error!("Search query error: {}", e);
                PlacesError::DatabaseError(e.to_string())
            })?;

        let places: Vec<Place> = rows.into_iter().map(|r| r.to_place()).collect();

        log::info!(
            "Search completed: {} results, {} total (page {}/{})",
            places.len(),
            total,
            page,
            (total + limit - 1) / limit
        );

        Ok((places, total))
    }

    /// Update existing place
    /// DOCUMENTATION: Partial update - only provided fields are modified
    pub async fn update_place(
        pool: &PgPool,
        id: Uuid,
        req: &UpdatePlaceRequest,
    ) -> Result<Place, PlacesError> {
        // Verify place exists
        let _ = Self::get_by_id(pool, id).await?;

        let updated: (Uuid,) = sqlx::query_as(
            r#"
            UPDATE places
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                tags = COALESCE($3, tags),
                vibe_descriptor = COALESCE($4, vibe_descriptor),
                opening_hours = COALESCE($5, opening_hours),
                google_rating = COALESCE($6, google_rating),
                business_status = COALESCE($7, business_status),
                updated_at = NOW()
            WHERE id = $8
            RETURNING id
            "#,
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.tags)
        .bind(&req.vibe_descriptor)
        .bind(&req.opening_hours)
        .bind(req.google_rating)
        .bind(&req.business_status)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Update failed for place {}: {}", id, e);
            PlacesError::DatabaseError(e.to_string())
        })?;

        let place = Self::get_by_id(pool, updated.0).await?;

        log::info!("Updated place: {}", id);
        Ok(place)
    }

    /// Soft delete place
    /// DOCUMENTATION: Sets is_active=false instead of physical deletion
    pub async fn delete_place(pool: &PgPool, id: Uuid) -> Result<(), PlacesError> {
        let rows =
            sqlx::query("UPDATE places SET is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| {
                    log::error!("Delete failed for place {}: {}", id, e);
                    PlacesError::DatabaseError(e.to_string())
                })?
                .rows_affected();

        if rows == 0 {
            return Err(PlacesError::NotFound(id.to_string()));
        }

        log::info!("Deleted place: {}", id);
        Ok(())
    }

    /// Bulk insert places (used by Google sync)
    /// DOCUMENTATION: Efficiently insert multiple places
    /// Returns count of successfully inserted places
    #[allow(dead_code)]
    pub async fn bulk_insert(
        pool: &PgPool,
        places: Vec<CreatePlaceRequest>,
    ) -> Result<u64, PlacesError> {
        let mut count = 0;

        for place in places {
            // In a real high-perf scenario, we'd use COPY or build a large VALUES statement.
            // For now, loop inserts are safer for error handling per item.
            match Self::create_place(pool, &place).await {
                Ok(_) => count += 1,
                Err(e) => {
                    log::warn!("Failed to insert place {}: {}", place.name, e);
                }
            }
        }

        Ok(count)
    }
}
