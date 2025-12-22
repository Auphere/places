-- migrations/008_extend_varchar_limits.sql

-- DOCUMENTATION: Extends VARCHAR limits for fields that commonly exceed 20 characters
-- PURPOSE: Fix "value too long for type character varying(20)" errors
-- DEPENDENCIES: 001_create_places.sql

-- Extend phone field from VARCHAR(20) to VARCHAR(50)
-- Many international phone numbers with extensions exceed 20 chars
ALTER TABLE places ALTER COLUMN phone TYPE VARCHAR(50);

-- Extend postal_code from VARCHAR(10) to VARCHAR(20)
-- Some international postal codes can be longer
ALTER TABLE places ALTER COLUMN postal_code TYPE VARCHAR(20);

-- Add comment for documentation
COMMENT ON COLUMN places.phone IS 'Phone number with international format support (max 50 chars)';
COMMENT ON COLUMN places.postal_code IS 'Postal/ZIP code with international format support (max 20 chars)';

