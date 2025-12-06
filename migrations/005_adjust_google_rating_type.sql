-- migrations/005_adjust_google_rating_type.sql
--
-- PURPOSE: Align google_rating column with Rust f32/SQL REAL type to avoid decode errors.
-- CONTEXT: Column was previously NUMERIC (DECIMAL), but SQLx expects REAL (FLOAT4) when mapping to Option<f32>.

BEGIN;

-- Drop dependent materialized view and views so column type can change
DROP MATERIALIZED VIEW IF EXISTS places_for_search;
DROP VIEW IF EXISTS places_search_optimized;
DROP VIEW IF EXISTS places_analytics_view;
DROP VIEW IF EXISTS sync_history_view;

-- Adjust column type to match Rust f32 / SQL REAL
ALTER TABLE places
    ALTER COLUMN google_rating DROP DEFAULT,
    ALTER COLUMN google_rating TYPE REAL USING google_rating::REAL;

-- Recreate materialized view (same definition as in initial migration)
CREATE MATERIALIZED VIEW places_for_search AS
SELECT 
    id, name, type, google_rating, location, city, district,
    main_categories, tags, vibe_descriptor,
    search_vector, google_place_id, is_active
FROM places
WHERE is_active = TRUE;

-- Recreate indexes for the materialized view
CREATE INDEX idx_search_view_location ON places_for_search USING GIST(location);
CREATE INDEX idx_search_view_search_vector ON places_for_search USING GIN(search_vector);

-- Recreate dependent views (copied from migration 003)
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

CREATE OR REPLACE VIEW sync_history_view AS
SELECT 
    source,
    COUNT(*) as sync_count,
    MAX(completed_at) as last_sync,
    SUM(records_created) as total_created,
    SUM(records_updated) as total_updated,
    AVG(duration_seconds) as avg_duration_seconds,
    SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_count
FROM data_sync_log
WHERE status IN ('completed', 'failed')
GROUP BY source;

COMMIT;

