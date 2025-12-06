-- migrations/006_enrich_places_fields.sql
-- DOCUMENTATION: Enriches places table with additional Google Places data
-- PURPOSE: Add fields for better place categorization and filtering
-- DEPENDENCIES: 005_adjust_google_rating_type.sql

-- Add new fields for enhanced place information
ALTER TABLE places 
    ADD COLUMN IF NOT EXISTS google_place_url VARCHAR(500),
    ADD COLUMN IF NOT EXISTS price_level INT CHECK (price_level >= 0 AND price_level <= 4),
    ADD COLUMN IF NOT EXISTS cuisine_types TEXT[],
    ADD COLUMN IF NOT EXISTS suitable_for TEXT[],
    ADD COLUMN IF NOT EXISTS is_open_now BOOLEAN;

-- Add comments for new columns
COMMENT ON COLUMN places.google_place_url IS 'Google Maps URL for this place';
COMMENT ON COLUMN places.price_level IS 'Price level from Google: 0 (free) to 4 (very expensive)';
COMMENT ON COLUMN places.cuisine_types IS 'Cuisine types: italian, japanese, mexican, etc.';
COMMENT ON COLUMN places.suitable_for IS 'Suitable for: couples, families, groups, solo, budget';
COMMENT ON COLUMN places.is_open_now IS 'Whether the place is currently open (from Google)';

-- Create index for price filtering
CREATE INDEX IF NOT EXISTS idx_places_price_level ON places(price_level) WHERE price_level IS NOT NULL;

-- Create GIN index for cuisine_types array queries
CREATE INDEX IF NOT EXISTS idx_places_cuisine_types ON places USING GIN(cuisine_types) WHERE cuisine_types IS NOT NULL;

-- Create GIN index for suitable_for array queries
CREATE INDEX IF NOT EXISTS idx_places_suitable_for ON places USING GIN(suitable_for) WHERE suitable_for IS NOT NULL;

-- Update the search vector trigger to include cuisine types
DROP TRIGGER IF EXISTS trg_update_places_search ON places;

CREATE OR REPLACE FUNCTION update_places_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := to_tsvector(
        'english',
        NEW.name || ' ' ||
        COALESCE(NEW.description, '') || ' ' ||
        array_to_string(COALESCE(NEW.main_categories, ARRAY[]::TEXT[]), ' ') || ' ' ||
        array_to_string(COALESCE(NEW.cuisine_types, ARRAY[]::TEXT[]), ' ')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_places_search
BEFORE INSERT OR UPDATE ON places
FOR EACH ROW
EXECUTE FUNCTION update_places_search_vector();

-- Refresh materialized view to include new fields
DROP MATERIALIZED VIEW IF EXISTS places_for_search;

CREATE MATERIALIZED VIEW places_for_search AS
SELECT 
    id, name, type, google_rating, location, city, district,
    main_categories, cuisine_types, suitable_for, price_level,
    tags, vibe_descriptor,
    search_vector, google_place_id, is_active
FROM places
WHERE is_active = TRUE;

CREATE INDEX idx_search_view_location ON places_for_search USING GIST(location);
CREATE INDEX idx_search_view_search_vector ON places_for_search USING GIN(search_vector);
CREATE INDEX idx_search_view_price ON places_for_search(price_level) WHERE price_level IS NOT NULL;

