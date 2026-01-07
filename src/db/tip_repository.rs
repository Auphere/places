// src/db/tip_repository.rs
// DOCUMENTATION: Database operations for place tips (no rating)

use crate::errors::PlacesError;
use crate::models::{CreateTipRequest, Tip};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TipRepository;

impl TipRepository {
    pub async fn create_tip(pool: &PgPool, req: &CreateTipRequest) -> Result<Tip, PlacesError> {
        let inserted: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO place_tips (
                place_id, source, source_id, author, text, posted_at, like_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 0))
            ON CONFLICT (source, source_id)
            DO UPDATE SET
                author = COALESCE(EXCLUDED.author, place_tips.author),
                text = COALESCE(EXCLUDED.text, place_tips.text),
                posted_at = COALESCE(EXCLUDED.posted_at, place_tips.posted_at),
                like_count = GREATEST(place_tips.like_count, EXCLUDED.like_count),
                updated_at = NOW()
            RETURNING id
            "#,
        )
        .bind(req.place_id)
        .bind(&req.source)
        .bind(&req.source_id)
        .bind(&req.author)
        .bind(&req.text)
        .bind(req.posted_at)
        .bind(req.like_count)
        .fetch_one(pool)
        .await
        .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

        Self::get_by_id(pool, inserted.0).await
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Tip, PlacesError> {
        let tip = sqlx::query_as::<_, Tip>(
            r#"
            SELECT
                id, place_id, source, source_id, author, text, posted_at, like_count,
                created_at, updated_at
            FROM place_tips
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

        Ok(tip)
    }

    pub async fn get_tips_by_place(
        pool: &PgPool,
        place_id: &Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Tip>, PlacesError> {
        let limit = limit.unwrap_or(20).max(1).min(100);
        let tips = sqlx::query_as::<_, Tip>(
            r#"
            SELECT
                id, place_id, source, source_id, author, text, posted_at, like_count,
                created_at, updated_at
            FROM place_tips
            WHERE place_id = $1
            ORDER BY posted_at DESC NULLS LAST, created_at DESC
            LIMIT $2
            "#,
        )
        .bind(place_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| PlacesError::DatabaseError(e.to_string()))?;

        Ok(tips)
    }
}


