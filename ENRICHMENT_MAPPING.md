# Place Data Enrichment - Mapping Documentation

## Overview

This document describes how data from Google Places API is mapped to the internal Auphere places database schema. The goal is to maximize the information available for each place to enable intelligent filtering and recommendations.

## Data Sources

### Primary Source: Google Places API

1. **Nearby Search API**: Initial discovery of places in a geographic area
2. **Place Details API**: Comprehensive information about each place (called for every place)

### Fields Retrieved from Google

```
- place_id (unique identifier)
- name
- types[] (categories)
- geometry.location (lat/lng)
- formatted_address
- address_components[] (detailed address breakdown)
- rating (0-5)
- user_ratings_total
- price_level (0-4)
- business_status (OPERATIONAL, CLOSED_TEMPORARILY, etc.)
- opening_hours (weekday_text, open_now, periods)
- formatted_phone_number
- international_phone_number
- website
- url (Google Maps link)
- reviews[]
- photos[]
```

## Field Mapping Rules

### Basic Identity Fields

| Internal Field     | Google Source        | Notes                                                |
| ------------------ | -------------------- | ---------------------------------------------------- |
| `name`             | `name`               | Direct mapping                                       |
| `description`      | `NULL`               | Not from Google - reserved for future LLM enrichment |
| `google_place_id`  | `place_id`           | Used for deduplication                               |
| `google_place_url` | `url` or constructed | Direct link to Google Maps                           |

### Geographic Fields

| Internal Field | Google Source                     | Extraction Rule                                                                                            |
| -------------- | --------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `latitude`     | `geometry.location.lat`           | Direct                                                                                                     |
| `longitude`    | `geometry.location.lng`           | Direct                                                                                                     |
| `address`      | `formatted_address` or `vicinity` | Prefer formatted_address                                                                                   |
| `city`         | Function parameter                | Provided by sync job (e.g., "Zaragoza")                                                                    |
| `district`     | `address_components[]`            | Extract from types: `sublocality`, `sublocality_level_1`, `neighborhood`, or `administrative_area_level_3` |
| `postal_code`  | `address_components[]`            | Extract from type: `postal_code`                                                                           |

### Rating & Business Info

| Internal Field        | Google Source        | Notes                                                                    |
| --------------------- | -------------------- | ------------------------------------------------------------------------ |
| `google_rating`       | `rating`             | Float 0-5                                                                |
| `google_rating_count` | `user_ratings_total` | Integer                                                                  |
| `price_level`         | `price_level`        | Integer 0-4 (0=free, 1=cheap, 2=moderate, 3=expensive, 4=very expensive) |
| `business_status`     | `business_status`    | String: OPERATIONAL, CLOSED_TEMPORARILY, CLOSED_PERMANENTLY              |

### Category & Type Mapping

#### Main Type (`type` field)

Priority-based mapping from Google `types[]` array:

```rust
Priority order:
1. "restaurant" → "restaurant"
2. "bar" → "bar"
3. "night_club" / "nightclub" → "nightclub"
4. "cafe" → "cafe"
5. "museum" → "other"
6. "park" → "other"
7. "shopping_mall" → "other"
8. "lodging" → "other"
9. "food" / "meal_takeaway" / "meal_delivery" → "restaurant"
10. default → "other"
```

#### Main Categories (`main_categories[]`)

- Extract first 3 Google types
- Exclude: `point_of_interest`, `establishment`, `geocode`
- Example: `["restaurant", "italian_restaurant", "food"]`

#### Secondary Categories (`secondary_categories[]`)

- Extract types 4-8 from Google types
- Same exclusion rules as main categories

#### Cuisine Types (`cuisine_types[]`)

**For restaurants and cafes only**

##### From Google Types:

```
"italian_restaurant" → "italian"
"chinese_restaurant" → "chinese"
"japanese_restaurant" → "japanese"
"mexican_restaurant" → "mexican"
"indian_restaurant" → "indian"
"spanish_restaurant" → "spanish"
"french_restaurant" → "french"
"thai_restaurant" → "thai"
"american_restaurant" → "american"
"mediterranean_restaurant" → "mediterranean"
```

##### From Name Keywords:

```
"pizza" → "italian"
"sushi", "ramen" → "japanese"
"taco", "burrito" → "mexican"
"curry" → "indian"
"tapas", "paella" → "spanish"
"burger", "bbq" → "american"
"thai" → "thai"
"vietnamese" → "vietnamese"
"korean" → "korean"
"mediterranean" → "mediterranean"
```

#### Suitable For Tags (`suitable_for[]`)

Derived from place type and price level:

| Condition                        | Tags Added                        |
| -------------------------------- | --------------------------------- |
| Park, museum, tourist attraction | `families`, `solo`, `groups`      |
| Bar, nightclub                   | `groups`, `couples`               |
| Cafe                             | `solo`, `couples`, `groups`       |
| Restaurant                       | `couples`, `families`, `groups`   |
| Restaurant + price_level >= 3    | Remove `families`, keep `couples` |
| Restaurant + price_level <= 1    | Add `budget`                      |

### Opening Hours

| Internal Field  | Google Source            | Format                                                     |
| --------------- | ------------------------ | ---------------------------------------------------------- |
| `opening_hours` | `opening_hours` object   | Stored as JSONB with `weekday_text`, `periods`, `open_now` |
| `is_open_now`   | `opening_hours.open_now` | Boolean extracted for quick filtering                      |

**Example stored format:**

```json
{
  "open_now": true,
  "weekday_text": [
    "Monday: 9:00 AM – 10:00 PM",
    "Tuesday: 9:00 AM – 10:00 PM",
    ...
  ],
  "periods": [...]
}
```

### Contact Information

| Internal Field | Google Source                                            | Priority               |
| -------------- | -------------------------------------------------------- | ---------------------- |
| `phone`        | `formatted_phone_number` or `international_phone_number` | Prefer formatted       |
| `website`      | `website`                                                | Direct                 |
| `email`        | N/A                                                      | Not provided by Google |

## Data Quality Notes

### Fields Often NULL from Google

- `district` - Only available if Google has detailed address components
- `postal_code` - Only for addresses with postal codes
- `price_level` - Not all businesses report this
- `opening_hours` - Not all places provide hours
- `description` - We intentionally leave this NULL (not from Google)

### Why `description` is NULL

The `formatted_address` already contains the address. Duplicating it in `description` provides no value. Instead, `description` should be:

- Left empty initially
- Enriched later with LLM-generated descriptions based on reviews, categories, and other metadata
- Never auto-filled with the address

## Using Enriched Data for Queries

### Example Agent Queries and Filters

**Query:** "Restaurante italiano romántico y barato en Las Delicias que abra hoy por la noche"

**Filters Applied:**

```sql
WHERE
  type = 'restaurant'
  AND 'italian' = ANY(cuisine_types)
  AND 'couples' = ANY(suitable_for)
  AND price_level <= 1
  AND district ILIKE '%Las Delicias%'
  AND is_open_now = true
  AND opening_hours->>'open_now' = 'true'
```

**Query:** "Café tranquilo para trabajar solo"

**Filters:**

```sql
WHERE
  type = 'cafe'
  AND 'solo' = ANY(suitable_for)
```

**Query:** "Museo familiar en el centro"

**Filters:**

```sql
WHERE
  (type = 'other' AND 'museum' = ANY(main_categories))
  AND 'families' = ANY(suitable_for)
  AND district IN ('Centro', 'Casco Histórico', ...)
```

## Database Schema Enhancements

See migration `006_enrich_places_fields.sql` for:

- New columns: `google_place_url`, `price_level`, `cuisine_types`, `suitable_for`, `is_open_now`
- Indexes on: `price_level`, `cuisine_types` (GIN), `suitable_for` (GIN)
- Updated full-text search to include `cuisine_types`

## API Response Format

The public API (`/places/search`, `/places/{id}`) returns:

```json
{
  "id": "uuid",
  "name": "Restaurante El Tubo",
  "description": null,
  "type": "restaurant",
  "latitude": 41.6561,
  "longitude": -0.8773,
  "address": "Calle Alfonso I, 3, 50003 Zaragoza, Spain",
  "city": "Zaragoza",
  "district": "Centro",
  "postal_code": "50003",
  "phone": "+34 976 123 456",
  "website": "https://example.com",
  "google_place_id": "ChIJ...",
  "google_place_url": "https://maps.google.com/?cid=...",
  "google_rating": 4.5,
  "google_rating_count": 234,
  "price_level": 2,
  "main_categories": ["restaurant", "spanish_restaurant", "food"],
  "secondary_categories": ["bar"],
  "cuisine_types": ["spanish", "tapas"],
  "tags": {},
  "vibe_descriptor": {},
  "suitable_for": ["couples", "families", "groups"],
  "opening_hours": { ... },
  "is_open_now": true,
  "business_status": "OPERATIONAL",
  "is_subscribed": false,
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z",
  "primary_photo_url": "https://...",
  "primary_photo_thumbnail_url": "https://..."
}
```

**Note:** `search_vector` is never exposed in the API (internal field only).

## Debugging & Inspection

### Admin Endpoint for Raw Data

**GET** `/admin/places/{id}/raw`

Headers: `X-Admin-Token: <your-token>`

Returns:

```json
{
  "place_id": "uuid",
  "database_record": { ... all fields including search_vector ... },
  "google_data": { ... raw Google Places response ... },
  "note": "This endpoint exposes internal fields for debugging..."
}
```

Use this endpoint to:

- Compare database data vs. Google source
- Debug mapping issues
- Verify extraction logic

## Future Enhancements

1. **LLM-Generated Descriptions**: Use GPT/Claude to generate compelling descriptions from reviews and metadata
2. **Vibe Tags**: Analyze reviews to extract atmosphere tags (romantic, lively, quiet, etc.)
3. **Sentiment Analysis**: Parse reviews for sentiment and common themes
4. **Dynamic Tags**: Auto-tag places based on review content (dog-friendly, outdoor seating, etc.)
5. **Seasonal Hours**: Track and predict opening hours changes
6. **Real-time Occupancy**: Integrate with Google Popular Times

## Maintenance

- **Refresh Cycle**: Run sync jobs periodically to update ratings, hours, and status
- **Data Validation**: Monitor for places with missing critical fields (district, postal_code)
- **Type Mapping Updates**: Expand cuisine and type mappings as new patterns emerge
- **Schema Migrations**: Always use versioned migrations for schema changes

---

**Last Updated:** 2025-12-03  
**Version:** 1.0  
**Author:** Auphere Development Team
