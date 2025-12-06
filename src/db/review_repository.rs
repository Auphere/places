// src/db/review_repository.rs
// DOCUMENTATION: Review database operations
// PURPOSE: Handle CRUD operations for place reviews

use crate::errors::PlacesError;
use crate::models::{CreateReviewRequest, Review};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReviewRepository;

impl ReviewRepository {
    /// Create a new review
    /// DOCUMENTATION: Insert review from any source (Google, Trustpilot, etc.)
    pub async fn create_review(
        pool: &PgPool,
        req: &CreateReviewRequest,
    ) -> Result<Review, PlacesError> {
        let review = sqlx::query_as::<_, Review>(
            r#"
            INSERT INTO place_reviews (
                place_id, source, source_id, author, rating, text, posted_at,
                is_verified, has_photo
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (source, source_id) DO UPDATE
            SET
                rating = EXCLUDED.rating,
                text = EXCLUDED.text,
                updated_at = CURRENT_TIMESTAMP
            RETURNING *
            "#,
        )
        .bind(&req.place_id)
        .bind(&req.source)
        .bind(&req.source_id)
        .bind(&req.author)
        .bind(req.rating)
        .bind(&req.text)
        .bind(&req.posted_at)
        .bind(req.is_verified.unwrap_or(false))
        .bind(req.has_photo.unwrap_or(false))
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to create review: {}", e);
            PlacesError::DatabaseError(format!("Create review failed: {}", e))
        })?;

        Ok(review)
    }

    /// Get reviews for a place
    /// DOCUMENTATION: Fetch all reviews for a specific place, optionally filtered by source
    pub async fn get_reviews_by_place(
        pool: &PgPool,
        place_id: &Uuid,
        source: Option<&str>,
    ) -> Result<Vec<Review>, PlacesError> {
        let query = match source {
            Some(src) => sqlx::query_as::<_, Review>(
                r#"
                    SELECT * FROM place_reviews
                    WHERE place_id = $1 AND source = $2
                    ORDER BY posted_at DESC
                    "#,
            )
            .bind(place_id)
            .bind(src),
            None => sqlx::query_as::<_, Review>(
                r#"
                    SELECT * FROM place_reviews
                    WHERE place_id = $1
                    ORDER BY posted_at DESC
                    "#,
            )
            .bind(place_id),
        };

        let reviews = query.fetch_all(pool).await.map_err(|e| {
            log::error!("Failed to fetch reviews for place {}: {}", place_id, e);
            PlacesError::DatabaseError(format!("Fetch reviews failed: {}", e))
        })?;

        Ok(reviews)
    }

    /// Delete all reviews for a place from a specific source
    /// DOCUMENTATION: Remove all reviews from a specific source (useful for re-sync)
    #[allow(dead_code)]
    pub async fn delete_reviews_by_source(
        pool: &PgPool,
        place_id: &Uuid,
        source: &str,
    ) -> Result<u64, PlacesError> {
        let result = sqlx::query(
            r#"
            DELETE FROM place_reviews
            WHERE place_id = $1 AND source = $2
            "#,
        )
        .bind(place_id)
        .bind(source)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to delete reviews for place {}: {}", place_id, e);
            PlacesError::DatabaseError(format!("Delete reviews failed: {}", e))
        })?;

        Ok(result.rows_affected())
    }

    /// Get review statistics by source
    /// DOCUMENTATION: Get aggregated statistics for reviews grouped by source
    #[allow(dead_code)]
    pub async fn get_review_stats_by_place(
        pool: &PgPool,
        place_id: &Uuid,
    ) -> Result<Vec<(String, i64, f64)>, PlacesError> {
        let stats = sqlx::query_as::<_, (String, i64, f64)>(
            r#"
            SELECT 
                source,
                COUNT(*) as count,
                AVG(rating) as avg_rating
            FROM place_reviews
            WHERE place_id = $1
            GROUP BY source
            ORDER BY source
            "#,
        )
        .bind(place_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch review stats for place {}: {}", place_id, e);
            PlacesError::DatabaseError(format!("Fetch review stats failed: {}", e))
        })?;

        Ok(stats)
    }
}
