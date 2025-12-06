-- migrations/004_create_photos_table.sql

-- DOCUMENTATION: Creates table for storing place photos from multiple sources
-- PURPOSE: Store photo URLs and metadata from Google Places, Yelp, Instagram, etc.

CREATE TABLE place_photos (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    place_id UUID NOT NULL REFERENCES places(id) ON DELETE CASCADE,
    
    -- Source Tracking
    source VARCHAR(50) NOT NULL,
    source_photo_reference VARCHAR(500),
    CONSTRAINT unique_source_reference UNIQUE (source, source_photo_reference),
    
    -- Photo URLs
    photo_url TEXT NOT NULL,
    thumbnail_url TEXT,
    
    -- Metadata
    width INT,
    height INT,
    attribution TEXT,
    
    -- Ordering and Selection
    is_primary BOOLEAN DEFAULT FALSE,
    display_order INT DEFAULT 0,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT valid_photo_source CHECK (source IN (
        'google', 'trustpilot', 'yelp', 'tripadvisor', 'instagram', 'user_upload', 'owner_upload'
    ))
);

COMMENT ON TABLE place_photos IS 'Stores photos from Google Places, Yelp, Instagram, and other sources';

-- Indexes
CREATE INDEX idx_photos_place_id ON place_photos(place_id);
CREATE INDEX idx_photos_source ON place_photos(source);
CREATE INDEX idx_photos_is_primary ON place_photos(is_primary) WHERE is_primary = TRUE;
CREATE INDEX idx_photos_display_order ON place_photos(place_id, display_order);

