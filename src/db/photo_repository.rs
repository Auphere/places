// src/db/photo_repository.rs
// DOCUMENTATION: Photo database operations
// PURPOSE: Handle CRUD operations for place photos

use crate::errors::PlacesError;
use crate::models::{CreatePhotoRequest, Photo};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PhotoRepository;

impl PhotoRepository {
    /// Create a new photo
    /// DOCUMENTATION: Insert photo from any source (Google, Yelp, Instagram, etc.)
    pub async fn create_photo(
        pool: &PgPool,
        req: &CreatePhotoRequest,
    ) -> Result<Photo, PlacesError> {
        let photo = sqlx::query_as::<_, Photo>(
            r#"
            INSERT INTO place_photos (
                place_id, source, source_photo_reference, photo_url,
                thumbnail_url, width, height, attribution,
                is_primary, display_order
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (source, source_photo_reference) DO UPDATE
            SET photo_url = EXCLUDED.photo_url,
                thumbnail_url = COALESCE(EXCLUDED.thumbnail_url, place_photos.thumbnail_url),
                width = COALESCE(EXCLUDED.width, place_photos.width),
                height = COALESCE(EXCLUDED.height, place_photos.height),
                attribution = COALESCE(EXCLUDED.attribution, place_photos.attribution),
                is_primary = EXCLUDED.is_primary,
                display_order = EXCLUDED.display_order,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(&req.place_id)
        .bind(&req.source)
        .bind(&req.source_photo_reference)
        .bind(&req.photo_url)
        .bind(&req.thumbnail_url)
        .bind(req.width)
        .bind(req.height)
        .bind(&req.attribution)
        .bind(req.is_primary.unwrap_or(false))
        .bind(req.display_order.unwrap_or(0))
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to create photo: {}", e);
            PlacesError::DatabaseError(format!("Create photo failed: {}", e))
        })?;

        Ok(photo)
    }

    /// Get photos for a place
    /// DOCUMENTATION: Fetch all photos for a specific place, ordered by display_order
    pub async fn get_photos_by_place(
        pool: &PgPool,
        place_id: &Uuid,
        source: Option<&str>,
    ) -> Result<Vec<Photo>, PlacesError> {
        let query = match source {
            Some(src) => sqlx::query_as::<_, Photo>(
                r#"
                    SELECT * FROM place_photos
                    WHERE place_id = $1 AND source = $2
                    ORDER BY is_primary DESC, display_order ASC
                    "#,
            )
            .bind(place_id)
            .bind(src),
            None => sqlx::query_as::<_, Photo>(
                r#"
                    SELECT * FROM place_photos
                    WHERE place_id = $1
                    ORDER BY is_primary DESC, display_order ASC
                    "#,
            )
            .bind(place_id),
        };

        let photos = query.fetch_all(pool).await.map_err(|e| {
            log::error!("Failed to fetch photos for place {}: {}", place_id, e);
            PlacesError::DatabaseError(format!("Fetch photos failed: {}", e))
        })?;

        Ok(photos)
    }

    /// Delete all photos for a place from a specific source
    /// DOCUMENTATION: Remove all photos from a specific source (useful for re-sync)
    #[allow(dead_code)]
    pub async fn delete_photos_by_source(
        pool: &PgPool,
        place_id: &Uuid,
        source: &str,
    ) -> Result<u64, PlacesError> {
        let result = sqlx::query(
            r#"
            DELETE FROM place_photos
            WHERE place_id = $1 AND source = $2
            "#,
        )
        .bind(place_id)
        .bind(source)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete photos for place {}: {}", place_id, e);
            PlacesError::DatabaseError(format!("Delete photos failed: {}", e))
        })?;

        Ok(result.rows_affected())
    }

    /// Set primary photo
    /// DOCUMENTATION: Mark a specific photo as primary (unsets other primaries for the place)
    #[allow(dead_code)]
    pub async fn set_primary_photo(
        pool: &PgPool,
        place_id: &Uuid,
        photo_id: &Uuid,
    ) -> Result<(), PlacesError> {
        // First, unset all primaries for this place
        sqlx::query(
            r#"
            UPDATE place_photos
            SET is_primary = FALSE
            WHERE place_id = $1
            "#,
        )
        .bind(place_id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to unset primary photos: {}", e);
            PlacesError::DatabaseError(format!("Unset primary failed: {}", e))
        })?;

        // Then set the new primary
        sqlx::query(
            r#"
            UPDATE place_photos
            SET is_primary = TRUE
            WHERE id = $1 AND place_id = $2
            "#,
        )
        .bind(photo_id)
        .bind(place_id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to set primary photo: {}", e);
            PlacesError::DatabaseError(format!("Set primary failed: {}", e))
        })?;

        Ok(())
    }
}
