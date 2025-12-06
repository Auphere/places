-- migrations/003_create_audit_tables.sql

-- DOCUMENTATION: Create triggers and views for complete audit trail
-- PURPOSE: Automatic change tracking and convenient views for analytics

-- Trigger Function: Automatic audit logging
CREATE OR REPLACE FUNCTION audit_places_changes()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO places_audit (place_id, action, old_data, new_data, changed_at)
    VALUES (
        CASE WHEN TG_OP = 'DELETE' THEN OLD.id ELSE NEW.id END,
        TG_OP,
        CASE WHEN TG_OP = 'INSERT' THEN NULL ELSE row_to_json(OLD) END,
        CASE WHEN TG_OP = 'DELETE' THEN NULL ELSE row_to_json(NEW) END,
        CURRENT_TIMESTAMP
    );
    
    RETURN CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END;
END;
$$ LANGUAGE plpgsql;

-- Attach audit trigger to places table
CREATE TRIGGER trg_audit_places
AFTER INSERT OR UPDATE OR DELETE ON places
FOR EACH ROW
EXECUTE FUNCTION audit_places_changes();

-- Trigger: Update review timestamp
CREATE OR REPLACE FUNCTION update_reviews_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_reviews_timestamp
BEFORE UPDATE ON place_reviews
FOR EACH ROW
EXECUTE FUNCTION update_reviews_updated_at();

-- Convenience View: High-performance search view
-- Updated to use English column names
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

-- View: Analytics Dashboard Data
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

-- View: Sync History
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
