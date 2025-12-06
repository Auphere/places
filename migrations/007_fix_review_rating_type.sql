-- migrations/007_fix_review_rating_type.sql

-- DOCUMENTATION: Fix review rating column type to match Rust f32 type
-- PURPOSE: Change rating from DECIMAL(3,1) to REAL (FLOAT4) for compatibility with Rust f32

BEGIN;

-- 1. Drop dependent views
DROP VIEW IF EXISTS places_search_optimized;
DROP VIEW IF EXISTS places_analytics_view;

-- 2. Change rating column type from DECIMAL to REAL
ALTER TABLE place_reviews 
    ALTER COLUMN rating TYPE REAL USING rating::REAL;

-- Also fix sentiment_score if it exists
ALTER TABLE place_reviews 
    ALTER COLUMN sentiment_score TYPE REAL USING sentiment_score::REAL;

-- Re-apply constraints
ALTER TABLE place_reviews
    DROP CONSTRAINT IF EXISTS place_reviews_rating_check;

ALTER TABLE place_reviews
    ADD CONSTRAINT place_reviews_rating_check 
    CHECK (rating >= 1.0 AND rating <= 5.0);

COMMENT ON COLUMN place_reviews.rating IS 'Review rating from 1.0 to 5.0 (REAL type for Rust f32 compatibility)';
COMMENT ON COLUMN place_reviews.sentiment_score IS 'Sentiment analysis score from -1.0 to 1.0 (REAL type for Rust f32 compatibility)';

-- 3. Recreate dependent views
CREATE OR REPLACE VIEW places_search_optimized AS
SELECT 
    p.id,
    p.name,
    p.type,
    p.google_rating,
    p.google_rating_count,
    p.location,
    p.city,
    p.district,
    p.address,
    p.website,
    p.phone,
    p.main_categories,
    p.tags,
    p.vibe_descriptor,
    p.opening_hours,
    p.is_subscribed,
    p.search_vector,
    COUNT(pr.id) as review_count,
    AVG(pr.rating) as avg_review_rating
FROM places p
LEFT JOIN place_reviews pr ON p.id = pr.place_id
WHERE p.is_active = TRUE
GROUP BY p.id
ORDER BY p.google_rating DESC;

CREATE OR REPLACE VIEW places_analytics_view AS
SELECT 
    p.id,
    p.name,
    p.city,
    p.district,
    p.type,
    p.is_subscribed,
    pm.monthly_impressions,
    pm.monthly_clicks,
    pm.monthly_plans_included,
    pm.reservations_completed,
    pm.estimated_occupancy,
    COUNT(pr.id) as total_reviews,
    AVG(pr.rating)::DECIMAL(3,1) as avg_rating
FROM places p
LEFT JOIN place_metrics pm ON p.id = pm.place_id
LEFT JOIN place_reviews pr ON p.id = pr.place_id
WHERE p.is_active = TRUE
GROUP BY p.id, pm.id;

COMMIT;
