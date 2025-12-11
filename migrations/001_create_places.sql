-- migrations/001_create_places.sql

-- DOCUMENTATION: Creates the main places table with geographic and metadata support
-- PURPOSE: Stores all place information with PostGIS integration for geographic queries
-- DEPENDENCIES: PostgreSQL 17+, PostGIS extension, UUID extension

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

CREATE TABLE IF NOT EXISTS places (
    -- Primary Identifier
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Basic Information (Required)
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL,
    
    -- Geographic Information
    location GEOMETRY(POINT, 4326) NOT NULL,
    address VARCHAR(500),
    city VARCHAR(100) NOT NULL,
    district VARCHAR(100), -- zona -> district/neighborhood
    postal_code VARCHAR(10),
    
    -- Contact Information
    phone VARCHAR(20),
    email VARCHAR(255),
    website VARCHAR(500),
    
    -- Google Places Integration
    google_place_id VARCHAR(255) UNIQUE,
    google_rating DECIMAL(3,1),
    google_rating_count INT DEFAULT 0,
    
    -- Classification & Tagging
    main_categories TEXT[], -- categories_principales
    secondary_categories TEXT[], -- categories_secundarias
    tags JSONB DEFAULT '{}',
    vibe_descriptor JSONB DEFAULT '{}',
    
    -- Operating Hours
    opening_hours JSONB DEFAULT '{}', -- horarios
    
    -- B2B Subscription (For restaurant owners)
    is_subscribed BOOLEAN DEFAULT FALSE,
    subscription_tier VARCHAR(50),
    subscription_expires_at TIMESTAMPTZ,
    owner_id UUID,
    
    -- Status Tracking
    is_active BOOLEAN DEFAULT TRUE,
    business_status VARCHAR(50),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_verified_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT valid_type CHECK (type IN (
        'restaurant', 'bar', 'cafe', 'club', 'nightclub', 'pub', 'lounge', 'bistro', 'other'
    )),
    CONSTRAINT valid_subscription_tier CHECK (subscription_tier IS NULL OR subscription_tier IN (
        'free', 'pro', 'enterprise'
    ))
);

-- COMMENTS
COMMENT ON TABLE places IS 'Stores all place information with PostGIS integration';
COMMENT ON COLUMN places.location IS 'Geographic coordinates (PostGIS)';
COMMENT ON COLUMN places.google_place_id IS 'Unique identifier from Google Places API';

-- Spatial Index (for geographic queries)
CREATE INDEX IF NOT EXISTS idx_places_location ON places USING GIST(location);

-- Categorical Indexes (for filtering)
CREATE INDEX IF NOT EXISTS idx_places_city ON places(city);
CREATE INDEX IF NOT EXISTS idx_places_district ON places(district);
CREATE INDEX IF NOT EXISTS idx_places_type ON places(type);

-- Identification Indexes
CREATE INDEX IF NOT EXISTS idx_places_google_id ON places(google_place_id);

-- Status Indexes (highly selective queries)
CREATE INDEX IF NOT EXISTS idx_places_active ON places(is_active);
CREATE INDEX IF NOT EXISTS idx_places_subscribed ON places(is_subscribed);

-- Composite Indexes (for common combined filters)
CREATE INDEX IF NOT EXISTS idx_places_city_type ON places(city, type);
CREATE INDEX IF NOT EXISTS idx_places_district_type ON places(district, type);

-- FULL-TEXT SEARCH DOCUMENTATION:
-- Implements PostgreSQL native full-text search (FTS) with English language support
-- (Note: can be switched to 'spanish' if the content is predominantly Spanish, but code/config remains English)

-- Add full-text search vector column
ALTER TABLE places ADD COLUMN IF NOT EXISTS search_vector TSVECTOR;

-- Initialize search vectors for existing records
UPDATE places SET search_vector = to_tsvector(
    'english', -- Changed to english config, though content might be mixed.
    name || ' ' ||
    COALESCE(description, '') || ' ' ||
    array_to_string(COALESCE(main_categories, ARRAY[]::TEXT[]), ' ')
);

-- Create GIN index for full-text search (logarithmic performance)
CREATE INDEX IF NOT EXISTS idx_places_search ON places USING GIN(search_vector);

-- Trigger function to maintain search_vector on every insert/update
CREATE OR REPLACE FUNCTION update_places_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := to_tsvector(
        'english',
        NEW.name || ' ' ||
        COALESCE(NEW.description, '') || ' ' ||
        array_to_string(COALESCE(NEW.main_categories, ARRAY[]::TEXT[]), ' ')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger to maintain search vector
DROP TRIGGER IF EXISTS trg_update_places_search ON places;
CREATE TRIGGER trg_update_places_search
BEFORE INSERT OR UPDATE ON places
FOR EACH ROW
EXECUTE FUNCTION update_places_search_vector();

-- JSONB Index for tag-based queries
CREATE INDEX IF NOT EXISTS idx_places_tags ON places USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_places_vibe ON places USING GIN(vibe_descriptor);

-- Filtered Index: Active places only (significant performance improvement)
CREATE INDEX IF NOT EXISTS idx_places_active_search ON places(search_vector)
    WHERE is_active = TRUE;

-- Update trigger to maintain updated_at timestamp
CREATE OR REPLACE FUNCTION update_places_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_update_places_updated_at ON places;
CREATE TRIGGER trg_update_places_updated_at
BEFORE UPDATE ON places
FOR EACH ROW
EXECUTE FUNCTION update_places_updated_at();

-- Materialized View for search optimization
DROP MATERIALIZED VIEW IF EXISTS places_for_search;
CREATE MATERIALIZED VIEW places_for_search AS
SELECT 
    id, name, type, google_rating, location, city, district,
    main_categories, tags, vibe_descriptor,
    search_vector, google_place_id, is_active
FROM places
WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_search_view_location ON places_for_search USING GIST(location);
CREATE INDEX IF NOT EXISTS idx_search_view_search_vector ON places_for_search USING GIN(search_vector);
