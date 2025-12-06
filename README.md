# Auphere Places Microservice

High-performance Rust microservice for managing Auphere place data. Features PostgreSQL (PostGIS) integration, full-text search, geographic proximity queries, and Google Places API synchronization.

## 🏗️ Architecture

- **Language:** Rust (2021 edition)
- **Framework:** Actix-web 4.x
- **Database:** PostgreSQL 17+ with PostGIS extension
- **Async Runtime:** Tokio
- **Database Access:** SQLx (Type-safe SQL queries)
- **External APIs:** Google Places API (only for manual sync)

## ✨ Features

### Core Features

- ✅ **CRUD Operations** - Full create, read, update, delete for places
- ✅ **Full-Text Search** - Native PostgreSQL FTS with English language support
- ✅ **Geographic Search** - Find places within radius using PostGIS
- ✅ **Advanced Filtering** - Search by city, district, type, rating, and custom tags
- ✅ **Pagination** - Cursor-based pagination for large result sets
- ✅ **Deduplication** - Prevents duplicate entries via Google Place ID

### Data Synchronization

- ✅ **Google Places Integration** - Automatic data import from Google Places API
- ✅ **Grid-Based Coverage** - Systematic city coverage using geographic grid
- ✅ **Batch Operations** - Sync multiple cities in one operation
- ✅ **Progress Tracking** - Detailed sync statistics and error reporting

### Data Management

- ✅ **Audit Logging** - Automatic tracking of all data changes
- ✅ **Soft Deletes** - Safe deletion with ability to restore
- ✅ **Multi-Source Reviews** - Support for Google, Yelp, TripAdvisor reviews
- ✅ **B2B Analytics** - Metrics tracking for business owners

## 📁 Project Structure

```
auphere-places/
├── src/
│   ├── config/                    # Configuration management
│   │   ├── db.rs                  # Database pool initialization
│   │   ├── env.rs                 # Environment variables
│   │   └── mod.rs
│   ├── db/                        # Data access layer
│   │   ├── repository.rs          # SQL queries and CRUD operations
│   │   └── mod.rs
│   ├── handlers/                  # HTTP request handlers
│   │   ├── health.rs              # Health check endpoint
│   │   ├── places.rs              # Place CRUD endpoints
│   │   ├── admin.rs               # Admin & sync endpoints
│   │   └── mod.rs
│   ├── models/                    # Data structures
│   │   ├── place.rs               # Place models and DTOs
│   │   └── mod.rs
│   ├── services/                  # Business logic
│   │   ├── place_service.rs       # Place operations
│   │   ├── google_places_client.rs # Google API client
│   │   ├── grid_generator.rs      # Geographic grid generation
│   │   ├── sync_service.rs        # Data synchronization
│   │   └── mod.rs
│   ├── errors.rs                  # Error types and handling
│   └── main.rs                    # Application entry point
├── migrations/                    # Database migrations
│   ├── 001_create_places.sql      # Main places table
│   ├── 002_create_search_index.sql # Supporting tables
│   └── 003_create_audit_tables.sql # Audit triggers
├── .env.example                   # Environment template
├── Cargo.toml                     # Dependencies
└── README.md                      # This file
```

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable) - [Install](https://rustup.rs/)
- **PostgreSQL** 17+ - [Install](https://www.postgresql.org/download/)
- **PostGIS** Extension - Usually included with PostgreSQL
- **Google Places API Key** (optional, for manual sync) - [Get Key](https://developers.google.com/maps/documentation/places/web-service/get-api-key)

### Installation Steps

#### 1. Clone and Setup Environment

```bash
# Navigate to project directory
cd auphere-places

# Copy environment template
cp .env.example .env

# Edit .env with your configuration
# Required: DATABASE_URL
# Optional: GOOGLE_PLACES_API_KEY (for sync operations)
```

#### 2. Database Setup

```bash
# Create database
createdb places

# Option 1: Run migrations using the provided script (Recommended)
cd auphere-places
./run_migrations.sh

# Option 2: Run migrations manually with psql
psql -U postgres -d places -f migrations/001_create_places.sql
psql -U postgres -d places -f migrations/002_create_search_index.sql
psql -U postgres -d places -f migrations/003_create_audit_tables.sql
psql -U postgres -d places -f migrations/004_create_photos_table.sql

# Option 3: Using environment variables
export DB_HOST=localhost
export DB_PORT=5432
export DB_NAME=places
export DB_USER=postgres
export DB_PASSWORD=your_password
./run_migrations.sh
```

#### 3. Build and Run

```bash
# Development mode (with hot reload)
cargo watch -x run

# Or standard run
cargo run

# Production build
cargo build --release
./target/release/auphere-places
```

The service will start on `http://127.0.0.1:3001` by default.

## 📚 API Documentation

### Base URL

```
http://localhost:3001
```

### Health Check

#### `GET /health`

Check if service is running.

**Response:**

```json
{
  "status": "ok",
  "service": "auphere-places",
  "version": "0.1.0"
}
```

---

### Place Endpoints

#### `POST /places`

Create a new place.

**Request Body:**

```json
{
  "name": "La Taberna del Alabardero",
  "description": "Traditional Spanish cuisine",
  "type": "restaurant",
  "location": [-3.7038, 40.4168],
  "address": "Calle Felipe V, 6",
  "city": "Madrid",
  "district": "Centro",
  "phone": "+34 915 47 25 77",
  "website": "https://example.com",
  "google_place_id": "ChIJd8BlQ2BZwokRAFUEcm_qrcA",
  "main_categories": ["restaurant", "spanish_cuisine", "fine_dining"]
}
```

**Response:** `201 Created`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "La Taberna del Alabardero",
  "type": "restaurant",
  "location": [-3.7038, 40.4168],
  "city": "Madrid",
  "district": "Centro",
  "google_rating": null,
  "google_rating_count": null,
  "main_categories": ["restaurant", "spanish_cuisine"],
  "tags": {},
  "vibe_descriptor": {},
  "website": "https://example.com",
  "is_subscribed": false,
  "created_at": "2025-11-30T12:00:00Z"
}
```

#### `GET /places/{id}`

Retrieve place by ID.

**Response:** `200 OK`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "La Taberna del Alabardero",
  ...
}
```

#### `PUT /places/{id}`

Update place (partial update).

**Request Body:**

```json
{
  "name": "Updated Name",
  "google_rating": 4.5,
  "business_status": "OPERATIONAL"
}
```

**Response:** `200 OK`

#### `DELETE /places/{id}`

Soft delete a place.

**Response:** `204 No Content`

#### `GET /places/search`

Search places with filters.

**Query Parameters:**

- `q` (string) - Full-text search query
- `city` (string) - Filter by city
- `district` (string) - Filter by district/neighborhood
- `type` (string) - Filter by place type
- `lat` (float) - Latitude for proximity search
- `lon` (float) - Longitude for proximity search
- `radius_km` (float) - Search radius in kilometers
- `min_rating` (float) - Minimum rating (0-5)
- `page` (int) - Page number (default: 1)
- `limit` (int) - Results per page (default: 20, max: 100)

**Example:**

```
GET /places/search?q=restaurant&city=Madrid&min_rating=4.0&limit=10
```

**Response:** `200 OK`

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "La Taberna del Alabardero",
      ...
    }
  ],
  "total_count": 150,
  "page": 1,
  "limit": 10,
  "has_more": true
}
```

---

### Admin Endpoints

**Authentication:** All admin endpoints require `X-Admin-Token` header.

#### `POST /admin/sync/{city}`

Trigger Google Places sync for a city.

**Headers:**

```
X-Admin-Token: your-admin-token
```

**Request Body (optional):**

```json
{
  "place_type": "restaurant",
  "cell_size_km": 1.5,
  "radius_m": 1000
}
```

**Response:** `200 OK`

```json
{
  "city": "Madrid",
  "api_requests": 50,
  "places_retrieved": 1000,
  "places_created": 850,
  "places_skipped": 120,
  "places_failed": 30,
  "errors": [],
  "duration_seconds": 120,
  "started_at": "2025-11-30T12:00:00Z",
  "completed_at": "2025-11-30T12:02:00Z"
}
```

**Supported Cities (Testing Phase):**

- Zaragoza (only city enabled for initial testing)

**Other cities available (commented out in code):**

- Madrid, Barcelona, Valencia, Sevilla, Bilbao, Málaga

#### `POST /admin/sync/batch`

Sync multiple cities in batch.

**Request Body:**

```json
{
  "cities": ["Madrid", "Barcelona", "Valencia"],
  "place_type": "restaurant"
}
```

**Response:** `200 OK`

```json
{
  "summary": {
    "city": "Multiple Cities",
    "places_created": 2500,
    "places_skipped": 350,
    ...
  },
  "details": [
    { "city": "Madrid", ... },
    { "city": "Barcelona", ... },
    { "city": "Valencia", ... }
  ]
}
```

#### `GET /admin/sync/status`

Get sync status and database statistics.

**Response:** `200 OK`

```json
{
  "message": "Sync service operational",
  "total_places": 5000,
  "active_places": 4850,
  "recent_additions": 150
}
```

#### `GET /admin/stats`

Get detailed database statistics.

**Response:** `200 OK`

```json
{
  "places_by_type": [
    { "type": "restaurant", "count": 2500 },
    { "type": "bar", "count": 1200 }
  ],
  "places_by_city": [
    { "city": "Madrid", "count": 2000 },
    { "city": "Barcelona", "count": 1500 }
  ],
  "average_rating": 4.2
}
```

---

## 🔧 Configuration

### Environment Variables

See `.env.example` for all available configuration options.

**Required:**

- `DATABASE_URL` - PostgreSQL connection string
- `SERVER_ADDRESS` - Bind address (default: 127.0.0.1)
- `SERVER_PORT` - Port number (default: 3001)

**Optional:**

- `GOOGLE_PLACES_API_KEY` - For sync operations
- `ADMIN_TOKEN` - Authentication token for admin endpoints
- `DB_MAX_CONNECTIONS` - Connection pool size (default: 20)
- `LOG_LEVEL` - Logging verbosity (default: info)

### Database Configuration

The service uses SQLx for compile-time checked SQL queries. To update queries:

```bash
# Prepare query metadata (required for offline builds)
cargo sqlx prepare

# Check if queries are valid
cargo sqlx prepare --check
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_grid_generation

# Run integration tests
cargo test --test '*'
```

## 📊 Database Schema

### Main Tables

- **places** - Core place data with PostGIS geometry
- **place_reviews** - Multi-source review aggregation
- **place_metrics** - B2B analytics and metrics
- **places_audit** - Change tracking and audit log
- **data_sync_log** - Synchronization history

### Key Indexes

- **GIN** index on `search_vector` for full-text search
- **GIST** index on `location` for geographic queries
- **BTree** indexes on `city`, `district`, `type` for filtering
- Composite indexes for common query patterns

## 🔒 Security

### Authentication

- Admin endpoints require `X-Admin-Token` header
- Token configured via `ADMIN_TOKEN` environment variable
- Use strong random tokens in production:
  ```bash
  openssl rand -hex 32
  ```

### Database Security

- Use SSL for database connections in production
- Follow principle of least privilege for database users
- Regularly backup audit logs

## 🚀 Deployment

### Docker (Recommended)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/auphere-places /usr/local/bin/
CMD ["auphere-places"]
```

### Production Checklist

- [ ] Set `ENVIRONMENT=production`
- [ ] Configure strong `ADMIN_TOKEN`
- [ ] Use SSL for database connections
- [ ] Set `LOG_LEVEL=info` or `warn`
- [ ] Configure appropriate `DB_MAX_CONNECTIONS`
- [ ] Set up database backups
- [ ] Configure reverse proxy (nginx, traefik)
- [ ] Set up monitoring and alerting
- [ ] Enable rate limiting at proxy level

## 📈 Performance

### Expected Performance

- **Search queries:** < 50ms (with proper indexes)
- **Geographic queries:** < 100ms (PostGIS optimized)
- **CRUD operations:** < 10ms
- **Concurrent requests:** 1000+ req/sec (with tuning)

### Optimization Tips

1. **Database Connection Pool:**

   - Tune `DB_MAX_CONNECTIONS` based on workload
   - Monitor connection pool utilization

2. **Indexes:**

   - All critical indexes are created by migrations
   - Consider materialized views for complex queries

3. **Caching:**
   - Consider Redis for frequently accessed data
   - Cache search results with short TTL

## 🐛 Troubleshooting

### Common Issues

**Database connection fails:**

```bash
# Check PostgreSQL is running
pg_isready

# Verify credentials
psql $DATABASE_URL

# Check PostGIS extension
psql -d places -c "SELECT PostGIS_version();"
```

**Migrations fail:**

```bash
# Reset database (CAUTION: destroys data)
sqlx database drop
sqlx database create
sqlx migrate run
```

**Google Places API errors:**

- Verify API key is valid
- Check API is enabled in Google Cloud Console
- Monitor quota usage

## 📝 License

[Your License Here]

## 🤝 Contributing

[Contribution guidelines if open source]

## 📧 Support

For issues and questions:

- GitHub Issues: [link]
- Email: [contact]
- Documentation: [link]

---

**Built with ❤️ using Rust**
