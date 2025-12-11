-- migrations/002_create_search_index.sql

-- DOCUMENTATION: Creates supporting tables for reviews, auditing, and metrics
-- PURPOSE: Maintains historical data, multi-source reviews, and B2B analytics

-- Audit Table: Track all changes to places
CREATE TABLE IF NOT EXISTS places_audit (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    place_id UUID NOT NULL REFERENCES places(id) ON DELETE CASCADE,
    
    action VARCHAR(50) NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    old_data JSONB,
    new_data JSONB,
    
    changed_by UUID,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Additional metadata
    change_reason VARCHAR(255),
    ip_address INET
);

COMMENT ON TABLE places_audit IS 'Immutable audit log for all place modifications';

-- Audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_place_id ON places_audit(place_id);
CREATE INDEX IF NOT EXISTS idx_audit_changed_at ON places_audit(changed_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_action ON places_audit(action);


-- Reviews Table: Aggregated from multiple sources
CREATE TABLE IF NOT EXISTS place_reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    place_id UUID NOT NULL REFERENCES places(id) ON DELETE CASCADE,
    
    -- Source Tracking
    source VARCHAR(50) NOT NULL,
    source_id VARCHAR(255),
    
    -- Review Content
    author VARCHAR(255),
    rating DECIMAL(3,1) NOT NULL CHECK (rating >= 1 AND rating <= 5),
    text TEXT,
    posted_at TIMESTAMPTZ NOT NULL,
    
    -- Sentiment Analysis (can be computed by AI)
    sentiment VARCHAR(20) CHECK (sentiment IN ('positive', 'neutral', 'negative')),
    sentiment_score DECIMAL(3,2),
    
    -- NLP Extracted Information
    extracted_tags JSONB,
    
    -- Engagement
    helpful_count INT DEFAULT 0,
    response_from_owner TEXT,
    
    -- Metadata
    is_verified BOOLEAN DEFAULT FALSE,
    has_photo BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints for data integrity
    UNIQUE(source, source_id),
    CONSTRAINT valid_source CHECK (source IN (
        'google', 'trustpilot', 'yelp', 'tripadvisor', 'instagram', 'custom'
    ))
);

COMMENT ON TABLE place_reviews IS 'Stores reviews from Google, Trustpilot, Yelp, etc.';

-- Review indexes
CREATE INDEX IF NOT EXISTS idx_reviews_place_id ON place_reviews(place_id);
CREATE INDEX IF NOT EXISTS idx_reviews_source ON place_reviews(source);
CREATE INDEX IF NOT EXISTS idx_reviews_rating ON place_reviews(rating DESC);
CREATE INDEX IF NOT EXISTS idx_reviews_posted_at ON place_reviews(posted_at DESC);
CREATE INDEX IF NOT EXISTS idx_reviews_sentiment ON place_reviews(sentiment);


-- Metrics Table: B2B analytics for place owners
CREATE TABLE IF NOT EXISTS place_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    place_id UUID NOT NULL REFERENCES places(id) ON DELETE CASCADE,
    
    -- Visibility Metrics
    monthly_impressions INT DEFAULT 0,
    monthly_clicks INT DEFAULT 0,
    monthly_plans_included INT DEFAULT 0,
    
    -- Conversion Metrics
    reservations_attempted INT DEFAULT 0,
    reservations_completed INT DEFAULT 0,
    reservations_no_show INT DEFAULT 0,
    
    -- Occupancy Data
    estimated_occupancy DECIMAL(5,2),
    occupancy_updated_at TIMESTAMPTZ,
    
    actual_visitors INT,
    actual_visitors_updated_at TIMESTAMPTZ,
    
    -- Time Period
    period_date DATE NOT NULL,
    metric_hour INT CHECK (metric_hour IS NULL OR (metric_hour >= 0 AND metric_hour <= 23)),
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(place_id, period_date),
    CONSTRAINT valid_metrics CHECK (
        monthly_impressions >= 0 AND
        monthly_clicks >= 0 AND
        reservations_attempted >= 0
    )
);

COMMENT ON TABLE place_metrics IS 'Daily/hourly metrics for dashboard and analytics';

-- Metrics indexes
CREATE INDEX IF NOT EXISTS idx_metrics_place_id ON place_metrics(place_id, period_date DESC);
CREATE INDEX IF NOT EXISTS idx_metrics_period ON place_metrics(period_date DESC);


-- Data Sync Log: Track Google Places and external source synchronization
CREATE TABLE IF NOT EXISTS data_sync_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Sync Identification
    source VARCHAR(100) NOT NULL,
    sync_type VARCHAR(50) NOT NULL,
    city VARCHAR(100),
    
    -- Statistics
    records_requested INT,
    records_processed INT,
    records_failed INT,
    records_created INT,
    records_updated INT,
    
    -- Error Tracking
    error_message TEXT,
    error_count INT DEFAULT 0,
    
    -- Timing Information
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,
    duration_seconds INT GENERATED ALWAYS AS (
        EXTRACT(EPOCH FROM (completed_at - started_at))::INT
    ) STORED,
    
    -- Scheduling
    next_sync_scheduled_for TIMESTAMPTZ,
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending', 'in_progress', 'completed', 'failed', 'partial'
    )),
    
    -- Metadata
    api_quota_used INT,
    api_quota_limit INT,
    notes TEXT
);

COMMENT ON TABLE data_sync_log IS 'Audit trail for all data synchronization operations';

-- Sync log indexes
CREATE INDEX IF NOT EXISTS idx_sync_log_source ON data_sync_log(source);
CREATE INDEX IF NOT EXISTS idx_sync_log_status ON data_sync_log(status);
CREATE INDEX IF NOT EXISTS idx_sync_log_started_at ON data_sync_log(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_sync_log_completed ON data_sync_log(completed_at DESC)
    WHERE status = 'completed';

