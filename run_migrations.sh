#!/bin/bash

# run_migrations.sh
# DOCUMENTATION: Script to run all database migrations in order
# PURPOSE: Apply all SQL migrations to the PostgreSQL database

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Load .env file if it exists (in project root)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"

if [ -f "$ENV_FILE" ]; then
    print_info "Loading environment variables from .env file..."
    # Export variables from .env (simple parser, handles KEY=VALUE format)
    set -a
    source "$ENV_FILE" 2>/dev/null || true
    set +a
fi

# Database connection parameters (can be overridden by environment variables)
# Try to parse DATABASE_URL if available, otherwise use individual variables
if [ -n "$DATABASE_URL" ]; then
    # Parse DATABASE_URL format: postgresql://user:password@host:port/dbname
    DB_USER="${DB_USER:-$(echo "$DATABASE_URL" | sed -n 's|.*://\([^:]*\):.*|\1|p')}"
    DB_PASSWORD="${DB_PASSWORD:-$(echo "$DATABASE_URL" | sed -n 's|.*://[^:]*:\([^@]*\)@.*|\1|p')}"
    DB_HOST="${DB_HOST:-$(echo "$DATABASE_URL" | sed -n 's|.*@\([^:]*\):.*|\1|p')}"
    DB_PORT="${DB_PORT:-$(echo "$DATABASE_URL" | sed -n 's|.*:\([0-9]*\)/.*|\1|p')}"
    DB_NAME="${DB_NAME:-$(echo "$DATABASE_URL" | sed -n 's|.*/\([^?]*\).*|\1|p')}"
fi

# Fallback to defaults if not set
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-places}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-}"

# Migration directory
MIGRATIONS_DIR="$(dirname "$0")/migrations"

# Check if psql is available
if ! command -v psql &> /dev/null; then
    print_error "psql command not found. Please install PostgreSQL client tools."
    exit 1
fi

# Build connection string
if [ -n "$DB_PASSWORD" ]; then
    export PGPASSWORD="$DB_PASSWORD"
    CONN_STRING="postgresql://${DB_USER}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
else
    CONN_STRING="postgresql://${DB_USER}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
fi

# Test database connection
print_info "Testing database connection..."
if ! psql "$CONN_STRING" -c "SELECT 1;" > /dev/null 2>&1; then
    print_error "Failed to connect to database."
    print_info "Connection details:"
    print_info "  Host: $DB_HOST"
    print_info "  Port: $DB_PORT"
    print_info "  Database: $DB_NAME"
    print_info "  User: $DB_USER"
    print_warn "Tip: Set environment variables to override defaults:"
    print_warn "  export DB_HOST=localhost"
    print_warn "  export DB_PORT=5432"
    print_warn "  export DB_NAME=places"
    print_warn "  export DB_USER=postgres"
    print_warn "  export DB_PASSWORD=your_password"
    exit 1
fi

print_info "Database connection successful!"

# Check if migrations directory exists
if [ ! -d "$MIGRATIONS_DIR" ]; then
    print_error "Migrations directory not found: $MIGRATIONS_DIR"
    exit 1
fi

# Get list of migration files in order
MIGRATIONS=(
    "001_create_places.sql"
    "002_create_search_index.sql"
    "003_create_audit_tables.sql"
    "004_create_photos_table.sql"
    "005_adjust_google_rating_type.sql"
    "006_enrich_places_fields.sql"
    "007_fix_review_rating_type.sql"
)

print_info "Found ${#MIGRATIONS[@]} migration(s) to apply"
echo ""

# Track migration status
SUCCESS_COUNT=0
FAILED_COUNT=0

# Apply each migration
for migration in "${MIGRATIONS[@]}"; do
    migration_path="${MIGRATIONS_DIR}/${migration}"
    
    if [ ! -f "$migration_path" ]; then
        print_warn "Migration file not found: $migration_path"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi
    
    print_info "Applying migration: $migration"
    
    if psql "$CONN_STRING" -f "$migration_path" > /dev/null 2>&1; then
        print_info "✓ Successfully applied: $migration"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        print_error "✗ Failed to apply: $migration"
        print_warn "Attempting to show error details..."
        psql "$CONN_STRING" -f "$migration_path" 2>&1 | tail -20
        FAILED_COUNT=$((FAILED_COUNT + 1))
        
        # Ask if user wants to continue
        read -p "Continue with remaining migrations? (y/n): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_warn "Migration process stopped by user."
            break
        fi
    fi
    
    echo ""
done

# Summary
echo "=========================================="
print_info "Migration Summary:"
print_info "  Successful: $SUCCESS_COUNT"
if [ $FAILED_COUNT -gt 0 ]; then
    print_error "  Failed: $FAILED_COUNT"
else
    print_info "  Failed: $FAILED_COUNT"
fi
echo "=========================================="

# Verify tables were created
print_info "Verifying database schema..."
TABLE_COUNT=$(psql "$CONN_STRING" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';" 2>/dev/null | xargs)

if [ -n "$TABLE_COUNT" ] && [ "$TABLE_COUNT" -gt 0 ]; then
    print_info "✓ Found $TABLE_COUNT table(s) in database"
    
    # List main tables
    print_info "Main tables:"
    psql "$CONN_STRING" -c "\dt" 2>/dev/null | grep -E "places|audit|reviews|photos|metrics|sync" || true
else
    print_warn "Could not verify tables. Please check manually."
fi

# Exit with appropriate code
if [ $FAILED_COUNT -gt 0 ]; then
    exit 1
else
    exit 0
fi

