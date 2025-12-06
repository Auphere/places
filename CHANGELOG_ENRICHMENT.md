# Changelog - Place Data Enrichment Update

## Version 2.0 - Enhanced Place Information (2025-12-03)

### Summary

This update significantly enriches the information stored and served for each place in the Auphere database. All data now comes directly from Google Places API with intelligent mapping and extraction of detailed information.

### Key Changes

#### 1. API Response Improvements

**FIXED:** API response structure now properly exposes useful fields:

**Before:**
```json
{
  "location": "0101000020E6100000...",  // WKB format - not useful
  "main_categories": "{lodging}",       // String instead of array
  "description": "Av. de Montañana, 564, ...",  // Duplicates address
  "search_vector": "'50059':8 '564':7..."      // Internal field exposed
}
```

**After:**
```json
{
  "latitude": 41.6970,
  "longitude": -0.8960,
  "main_categories": ["lodging"],
  "description": null,  // Reserved for future LLM enrichment
  // search_vector not exposed
}
```

#### 2. New Fields from Google Places

All fields now properly extracted from Google Places Details API:

| Field | Description | Source |
|-------|-------------|--------|
| `google_place_url` | Direct link to Google Maps | `url` from API |
| `price_level` | 0-4 (free to very expensive) | `price_level` from API |
| `district` | Neighborhood/barrio | Extracted from `address_components` |
| `postal_code` | Postal code | Extracted from `address_components` |
| `cuisine_types[]` | Italian, Japanese, etc. | Derived from `types` + name analysis |
| `suitable_for[]` | Couples, families, groups, solo, budget | Intelligent mapping from type + price |
| `is_open_now` | Currently open? | Extracted from `opening_hours.open_now` |
| `secondary_categories[]` | Additional categories | Extended `types` mapping |

#### 3. Intelligent Data Extraction

**District & Postal Code:**
- Parsed from Google's `address_components` array
- Looks for: `sublocality`, `neighborhood`, `administrative_area_level_3`
- Example: "Las Delicias", "Centro", "50003"

**Cuisine Types:**
- For restaurants/cafes: extracted from Google types + name keywords
- Types: `italian_restaurant` → `italian`
- Name: "Pizzeria Roma" → `italian`
- Name: "Sushi Bar" → `japanese`

**Suitable For Tags:**
- Parks/Museums → `families`, `solo`, `groups`
- Bars/Nightclubs → `groups`, `couples`
- Restaurants → `couples`, `families`, `groups`
- Expensive (price ≥ 3) → Remove `families`, emphasize `couples`
- Cheap (price ≤ 1) → Add `budget`

**Price Level Mapping:**
```
0 = Free
1 = Inexpensive (€)
2 = Moderate (€€)
3 = Expensive (€€€)
4 = Very Expensive (€€€€)
```

#### 4. Description Field Fixed

**Problem:** `description` was duplicating the `address` field.

**Solution:** 
- Set to `NULL` during sync
- Reserved for future LLM-generated descriptions
- Will use GPT/Claude to create compelling descriptions from reviews and metadata

#### 5. Database Schema Updates

**New Migration:** `006_enrich_places_fields.sql`

```sql
ALTER TABLE places ADD COLUMN
  google_place_url VARCHAR(500),
  price_level INT CHECK (price_level >= 0 AND price_level <= 4),
  cuisine_types TEXT[],
  suitable_for TEXT[],
  is_open_now BOOLEAN;
```

**New Indexes:**
- `idx_places_price_level` - For price filtering
- `idx_places_cuisine_types` (GIN) - For cuisine searches
- `idx_places_suitable_for` (GIN) - For "suitable for" queries

**Updated Full-Text Search:**
- Now includes `cuisine_types` in search vector
- Better relevance for food-related searches

#### 6. Enhanced Google Places Integration

**Before:** Only called Nearby Search (basic info)

**After:** 
1. Nearby Search for discovery
2. **Place Details API** called for every place
3. Comprehensive field extraction:
   - `address_components` for location details
   - `price_level` for cost indication
   - `opening_hours` with full schedule
   - `reviews` and `photos` saved to separate tables

#### 7. New Admin Debug Endpoint

**GET** `/admin/places/{id}/raw`

Returns:
```json
{
  "place_id": "uuid",
  "database_record": { /* All DB fields including internal ones */ },
  "google_data": { /* Fresh data from Google Places API */ },
  "note": "For debugging mapping issues"
}
```

**Use Cases:**
- Compare what's in DB vs. what Google returns
- Debug why district/postal_code is NULL
- Verify category mapping
- Inspect raw Google types

### Migration Instructions

#### 1. Run Database Migration

```bash
cd auphere-places
./run_migrations.sh
```

This will apply migration `006_enrich_places_fields.sql`.

#### 2. Rebuild & Restart Service

```bash
# Recompile with new schema
cargo build --release

# Restart service
cargo run
```

#### 3. Re-sync Places (Recommended)

To populate new fields for existing places:

```bash
cd ../auphere-places
python populate_zaragoza.py
```

**Note:** Existing places will be **updated** (not duplicated) thanks to `google_place_id` uniqueness constraint. The upsert logic will:
- Keep existing `id`, `created_at`
- Update all other fields with fresh Google data
- Populate new fields: `district`, `postal_code`, `price_level`, `cuisine_types`, etc.

#### 4. Verify Enrichment

**Check a place:**
```bash
curl http://localhost:3001/places/search?city=Zaragoza&limit=1 | jq
```

**Expected output should now have:**
```json
{
  "data": [{
    "latitude": 41.65,
    "longitude": -0.87,
    "district": "Centro",
    "postal_code": "50001",
    "price_level": 2,
    "cuisine_types": ["spanish", "tapas"],
    "suitable_for": ["couples", "families", "groups"],
    "google_place_url": "https://maps.google.com/?cid=...",
    "is_open_now": true,
    ...
  }]
}
```

**Debug a specific place:**
```bash
curl -H "X-Admin-Token: your-token" \
  http://localhost:3001/admin/places/{id}/raw | jq
```

### Breaking Changes

⚠️ **API Response Structure Changed**

Old code expecting `location: [lng, lat]` should now use:
```
latitude: 41.65
longitude: -0.87
```

Old code expecting string categories:
```json
"main_categories": "{lodging}"
```

Should now expect arrays:
```json
"main_categories": ["lodging"]
```

### Performance Impact

- **Sync Time**: ~2x longer (due to Place Details API calls for each place)
- **API Cost**: +1 request per place (Place Details = $0.017/request)
- **Query Performance**: Improved with new GIN indexes on arrays
- **Storage**: ~5-10% increase due to new fields

### Cost Estimate

For 1000 places:
- **Before**: 10 API requests (grid cells) × $0.032 = **$0.32**
- **After**: 10 + 1000 (details) × $0.017 = **$17.32**

**Optimization:** Place Details calls include 100ms delay to respect rate limits.

### Agent Query Examples

These queries are now possible thanks to enriched data:

**1. "Restaurante italiano barato en Centro"**
```sql
WHERE type = 'restaurant'
  AND 'italian' = ANY(cuisine_types)
  AND price_level <= 1
  AND district = 'Centro'
```

**2. "Lugar romántico para parejas"**
```sql
WHERE 'couples' = ANY(suitable_for)
  AND price_level >= 2
```

**3. "Sitio abierto ahora en Las Delicias"**
```sql
WHERE is_open_now = true
  AND district ILIKE '%Las Delicias%'
```

**4. "Café tranquilo para trabajar solo"**
```sql
WHERE type = 'cafe'
  AND 'solo' = ANY(suitable_for)
```

### Documentation

See **ENRICHMENT_MAPPING.md** for complete field mapping rules and extraction logic.

### Future Roadmap

1. **LLM Descriptions** (Q1 2025)
   - Generate descriptions from reviews
   - Multilingual support (ES, EN)

2. **Vibe Tags** (Q1 2025)
   - Extract from reviews: "romantic", "lively", "quiet"
   - Sentiment analysis

3. **Dynamic Tags** (Q2 2025)
   - "dog-friendly", "outdoor seating", "wheelchair accessible"
   - Parse from reviews and photos

4. **Real-time Data** (Q2 2025)
   - Popular times integration
   - Live occupancy estimates

### Testing

Run integration tests:
```bash
cargo test --test integration_tests
```

Test sync with small area:
```bash
curl -X POST http://localhost:3001/admin/sync/Zaragoza \
  -H "X-Admin-Token: your-token" \
  -H "Content-Type: application/json" \
  -d '{"place_type": "restaurant", "cell_size_km": 0.5, "radius_m": 500}'
```

### Rollback Instructions

If you need to rollback:

```sql
-- Remove new columns
ALTER TABLE places 
  DROP COLUMN IF EXISTS google_place_url,
  DROP COLUMN IF EXISTS price_level,
  DROP COLUMN IF EXISTS cuisine_types,
  DROP COLUMN IF EXISTS suitable_for,
  DROP COLUMN IF EXISTS is_open_now;

-- Drop new indexes
DROP INDEX IF EXISTS idx_places_price_level;
DROP INDEX IF EXISTS idx_places_cuisine_types;
DROP INDEX IF EXISTS idx_places_suitable_for;
```

Then checkout previous commit and rebuild.

---

**Questions?** Check `ENRICHMENT_MAPPING.md` or open an issue.

**Feedback:** This is v2.0 of the enrichment system. Please report any mapping issues or missing fields.

