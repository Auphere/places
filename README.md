# Auphere Places

Microservicio de **Places** (Rust + Actix + Postgres/PostGIS) que centraliza:

- **Búsqueda** (`/places/search`) con Google Places (si hay API key) o DB fallback.
- **Detalle** (`/places/{id}`) con persistencia on-demand:
  - Google Place Details → guarda place + photos + reviews
  - Foursquare (opcional) → guarda photos + tips
- **Clustering por zonas** (`/places/clusters`) usando PostGIS DBSCAN (DB-only).

Para detalles completos de endpoints y comportamiento ver `SERVICES.md`.

## Requisitos

- Rust (toolchain)
- PostgreSQL con PostGIS
- `psql` instalado (para migraciones)

## Configuración (.env)

```env
DATABASE_URL=postgresql://USER:PASS@localhost:5432/places
SERVER_ADDRESS=0.0.0.0
SERVER_PORT=8002
ENVIRONMENT=development
LOG_LEVEL=info

# Opcionales
GOOGLE_PLACES_API_KEY=...
FOURSQUARE_API_KEY=...
```

## Migraciones

```bash
./run_migrations.sh
```

## Ejecutar

```bash
cargo run
```

## Health

```bash
curl http://localhost:8002/health
```
