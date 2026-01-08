# Auphere Places

Microservicio de **Places** (Rust + Actix + Postgres/PostGIS) que actúa como **Source of Truth (SoT)** para todos los datos de lugares.

## Funcionalidades

- **Búsqueda** (`/places/search`) con Google Places (si hay API key) o DB fallback
- **Detalle** (`/places/{id}`) con persistencia on-demand:
  - Google Place Details → guarda place + photos + reviews
  - Foursquare (opcional) → guarda photos + tips
- **Clustering por zonas** (`/places/clusters`) usando PostGIS DBSCAN
- **Cache inteligente** con stale-while-revalidate (SWR)

## Tecnologías

- **Framework:** Actix Web
- **Base de datos:** PostgreSQL + PostGIS
- **Lenguaje:** Rust
- **APIs externas:** Google Places, Foursquare

## Requisitos

- Rust (toolchain estable)
- PostgreSQL con PostGIS habilitado
- `psql` instalado (para migraciones)

## Configuración (.env)

```env
# Database
DATABASE_URL=postgresql://USER:PASS@localhost:5432/places

# Server
SERVER_ADDRESS=0.0.0.0
SERVER_PORT=8002
ENVIRONMENT=development
LOG_LEVEL=info

# APIs externas (opcionales)
GOOGLE_PLACES_API_KEY=your_google_api_key
FOURSQUARE_API_KEY=your_foursquare_api_key
```

## Migraciones

```bash
# Ejecutar todas las migraciones
./run_migrations.sh

# O manualmente
psql $DATABASE_URL -f migrations/001_create_places.sql
psql $DATABASE_URL -f migrations/002_create_search_index.sql
# ... etc
```

## Ejecutar

```bash
# Desarrollo
cargo run

# Release
cargo run --release

# Build
cargo build --release
```

## Verificación

```bash
# Health check
curl http://localhost:8002/health

# Búsqueda de prueba
curl "http://localhost:8002/places/search?city=Madrid&q=restaurantes"
```

## Endpoints principales

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/places/search` | Buscar lugares |
| GET | `/places/nearby` | Lugares cercanos |
| GET | `/places/{id}` | Detalle de lugar (con enrichment) |
| GET | `/places/clusters` | Clustering por zonas (PostGIS) |

## Docker

```bash
docker build -t auphere-places:latest .
docker run -p 8002:8002 --env-file .env auphere-places:latest
```

## Estructura de directorios

```
auphere-places/
├── src/
│   ├── main.rs           # Entry point
│   ├── config/           # Configuración y env
│   ├── handlers/         # HTTP handlers
│   ├── models/           # Modelos de datos
│   ├── db/               # Repositorios (Postgres)
│   └── services/         # Clientes externos (Google, Foursquare)
├── migrations/           # SQL migrations
├── Cargo.toml
└── Dockerfile
```
