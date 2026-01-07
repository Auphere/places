-- migrations/010_create_place_tips.sql
--
-- PURPOSE: Store "tips" (short text notes) from sources like Foursquare.
-- We keep tips separate from place_reviews because tips generally don't have a 1..5 rating.

CREATE TABLE IF NOT EXISTS place_tips (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    place_id UUID NOT NULL REFERENCES places(id) ON DELETE CASCADE,

    -- Source tracking (v1: only 'foursquare' used)
    source VARCHAR(50) NOT NULL,
    source_id VARCHAR(255),
    CONSTRAINT unique_tip_source UNIQUE (source, source_id),

    -- Tip content
    author VARCHAR(255),
    text TEXT,
    posted_at TIMESTAMPTZ,
    like_count INT DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT valid_tip_source CHECK (source IN ('foursquare', 'custom'))
);

COMMENT ON TABLE place_tips IS 'Stores short tips/notes from sources like Foursquare (no rating)';

CREATE INDEX IF NOT EXISTS idx_tips_place_id ON place_tips(place_id);
CREATE INDEX IF NOT EXISTS idx_tips_source ON place_tips(source);
CREATE INDEX IF NOT EXISTS idx_tips_posted_at ON place_tips(posted_at DESC);


