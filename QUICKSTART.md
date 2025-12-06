# Quick Start Guide - Auphere Places Microservice

Get the Auphere Places microservice running in 5 minutes!

## Prerequisites Check

Before starting, ensure you have:

```bash
# Check Rust installation
rustc --version
# Should output: rustc 1.75.0 (or later)

# Check PostgreSQL installation
psql --version
# Should output: psql (PostgreSQL) 17.0 (or later)

# Check cargo is available
cargo --version
```

If any of these are missing:

- **Rust**: Install from https://rustup.rs/
- **PostgreSQL 17+**: Install from https://www.postgresql.org/download/

## Step-by-Step Setup

### 1. Configure Environment (2 minutes)

```bash
# Navigate to the project directory
cd auphere-places

# Copy environment template
cp .env.example .env

# Edit the .env file with your settings
# Minimum required: DATABASE_URL
nano .env  # or use your preferred editor
```

**Minimal .env configuration:**

```env
DATABASE_URL=postgresql://yourusername:yourpassword@localhost:5432/places
SERVER_ADDRESS=127.0.0.1
SERVER_PORT=3001
ENVIRONMENT=development
LOG_LEVEL=debug
ADMIN_TOKEN=my-secret-admin-token
```

### 2. Setup Database (1 minute)

```bash
# Create the database
createdb places

# OR if you need to specify user/host
createdb -U yourusername -h localhost places

# Install SQLx CLI (one-time setup)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

**Verify migrations worked:**

```bash
psql places -c "\dt"
# Should show: places, places_audit, place_reviews, place_metrics, data_sync_log
```

### 3. Build and Run (2 minutes)

```bash
# Build and run (development mode)
cargo run

# You should see output like:
# [INFO] Starting auphere-places microservice...
# [INFO] Database pool initialized successfully
# [INFO] Server running at http://127.0.0.1:3001
```

### 4. Test the Service (30 seconds)

Open a new terminal and test:

```bash
# Test health endpoint
curl http://localhost:3001/health

# Expected response:
# {"status":"ok","service":"auphere-places","version":"0.1.0"}
```

**Success! 🎉** Your service is now running!

---

## Quick API Testing

### Create a Place

```bash
curl -X POST http://localhost:3001/places \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Restaurant",
    "description": "A test place",
    "type": "restaurant",
    "location": [-3.7038, 40.4168],
    "address": "Test Street 123",
    "city": "Madrid",
    "district": "Centro",
    "phone": "+34 123 456 789",
    "website": "https://test.com",
    "main_categories": ["restaurant", "spanish_cuisine"]
  }'
```

Save the `id` from the response for the next steps!

### Get Place by ID

```bash
# Replace {id} with the actual ID from the previous response
curl http://localhost:3001/places/{id}
```

### Search Places

```bash
# Search by city
curl "http://localhost:3001/places/search?city=Madrid"

# Search with text query
curl "http://localhost:3001/places/search?q=restaurant&city=Madrid"

# Search nearby (within 5km of Madrid center)
curl "http://localhost:3001/places/search?lat=40.4168&lon=-3.7038&radius_km=5"
```

### Update Place

```bash
curl -X PUT http://localhost:3001/places/{id} \
  -H "Content-Type: application/json" \
  -d '{
    "google_rating": 4.5,
    "business_status": "OPERATIONAL"
  }'
```

### Delete Place

```bash
# Soft delete (can be restored)
curl -X DELETE http://localhost:3001/places/{id}
```

---

## Google Places Sync (Optional)

If you have a Google Places API key:

### 1. Add API Key to .env

```env
GOOGLE_PLACES_API_KEY=YOUR_ACTUAL_API_KEY_HERE
```

### 2. Test Sync

```bash
# Sync restaurants in Zaragoza (only city enabled for testing)
curl -X POST http://localhost:3001/admin/sync/Zaragoza \
  -H "X-Admin-Token: my-secret-admin-token" \
  -H "Content-Type: application/json" \
  -d '{"place_type": "restaurant"}'
```

**Note**: This will make many API requests and may take 2-3 minutes. Currently only **Zaragoza** is enabled for testing.

### 3. Check Sync Status

```bash
curl http://localhost:3001/admin/sync/status \
  -H "X-Admin-Token: my-secret-admin-token"
```

### 4. View Statistics

```bash
curl http://localhost:3001/admin/stats \
  -H "X-Admin-Token: my-secret-admin-token"
```

---

## Common Issues & Solutions

### Issue: "Database connection failed"

```bash
# Test PostgreSQL connection
psql $DATABASE_URL

# If connection fails, check:
# 1. PostgreSQL is running: pg_isready
# 2. Credentials are correct
# 3. Database exists: psql -l
```

### Issue: "Migrations failed"

```bash
# Check current migration status
sqlx migrate info

# Revert last migration if needed
sqlx migrate revert

# Re-run migrations
sqlx migrate run
```

### Issue: "PostGIS not found"

```bash
# Install PostGIS extension
psql places -c "CREATE EXTENSION IF NOT EXISTS postgis;"
psql places -c "SELECT PostGIS_version();"
```

### Issue: "Port 3001 already in use"

```bash
# Change port in .env
SERVER_PORT=3002

# Or find and kill process using port 3001
lsof -ti:3001 | xargs kill -9
```

### Issue: "Google Places API error"

1. Verify API key is correct in `.env`
2. Check API is enabled in Google Cloud Console
3. Verify billing is enabled for your project
4. Check quota hasn't been exceeded

---

## Development Tips

### Watch Mode (Auto-reload on changes)

```bash
# Install cargo-watch
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
```

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_grid_generation
```

### Check Code

```bash
# Check for compilation errors
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

### View Logs

```bash
# More verbose logging
LOG_LEVEL=debug cargo run

# Or set in .env:
LOG_LEVEL=debug
RUST_LOG=debug,actix_web=debug,sqlx=debug
```

### Database Queries

```bash
# Connect to database
psql places

# View all places
places=# SELECT id, name, city, type FROM places LIMIT 5;

# Count places by city
places=# SELECT city, COUNT(*) FROM places GROUP BY city;

# View recent sync operations
places=# SELECT * FROM data_sync_log ORDER BY started_at DESC LIMIT 5;
```

---

## Next Steps

1. ✅ Read the full [README.md](README.md) for complete documentation
2. ✅ Check [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) for architecture details
3. ✅ Explore the code in `src/` directory
4. ✅ Review database schema in `migrations/` directory
5. ✅ Try syncing data from Google Places (if you have an API key)
6. ✅ Build your application on top of this microservice!

---

## API Endpoint Summary

| Method | Endpoint             | Purpose             | Auth Required       |
| ------ | -------------------- | ------------------- | ------------------- |
| GET    | `/health`            | Health check        | No                  |
| POST   | `/places`            | Create place        | No                  |
| GET    | `/places/{id}`       | Get place details   | No                  |
| PUT    | `/places/{id}`       | Update place        | No                  |
| DELETE | `/places/{id}`       | Delete place        | No                  |
| GET    | `/places/search`     | Search places       | No                  |
| POST   | `/admin/sync/{city}` | Sync city data      | Yes (X-Admin-Token) |
| POST   | `/admin/sync/batch`  | Batch sync cities   | Yes (X-Admin-Token) |
| GET    | `/admin/sync/status` | Get sync status     | Yes (X-Admin-Token) |
| GET    | `/admin/stats`       | Database statistics | Yes (X-Admin-Token) |

---

## Support

Having issues? Check these resources:

1. **README.md** - Full documentation
2. **IMPLEMENTATION_SUMMARY.md** - Technical details
3. **GitHub Issues** - Report bugs
4. **Rust Documentation** - https://doc.rust-lang.org/
5. **Actix Web Guide** - https://actix.rs/docs/

---

**Happy coding! 🚀**
