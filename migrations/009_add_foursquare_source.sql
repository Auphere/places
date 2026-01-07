-- migrations/009_add_foursquare_source.sql
--
-- PURPOSE: Allow Foursquare as a first-class data source for photos and reviews.
-- NOTE: We keep existing sources and only extend allowed values.
--
-- This enables persisting multi-source enrichment (Google + Foursquare) in auphere-places.

-- Extend place_photos source constraint to include 'foursquare'
ALTER TABLE place_photos
    DROP CONSTRAINT IF EXISTS valid_photo_source;

ALTER TABLE place_photos
    ADD CONSTRAINT valid_photo_source CHECK (source IN (
        'google', 'foursquare', 'trustpilot', 'yelp', 'tripadvisor', 'instagram', 'user_upload', 'owner_upload'
    ));

-- Extend place_reviews source constraint to include 'foursquare'
ALTER TABLE place_reviews
    DROP CONSTRAINT IF EXISTS valid_source;

ALTER TABLE place_reviews
    ADD CONSTRAINT valid_source CHECK (source IN (
        'google', 'foursquare', 'trustpilot', 'yelp', 'tripadvisor', 'instagram', 'custom'
    ));


