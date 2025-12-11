# 🗺️ Auphere Places

**High-Performance Places Microservice**

Microservicio de lugares construido en Rust con Actix-web y PostgreSQL/PostGIS para búsqueda y gestión de lugares de forma ultrarrápida y escalable.

---

## 📋 **Tabla de Contenidos**

- [Descripción](#descripción)
- [Tecnologías](#tecnologías)
- [Requisitos Previos](#requisitos-previos)
- [Instalación](#instalación)
- [Configuración](#configuración)
- [Ejecución](#ejecución)
- [Migraciones](#migraciones)
- [API Endpoints](#api-endpoints)
- [Testing](#testing)
- [Docker](#docker)
- [Troubleshooting](#troubleshooting)

---

## 📝 **Descripción**

El microservicio Places de Auphere proporciona:

- **Búsqueda ultrarrápida** de lugares con filtros avanzados
- **Búsqueda geoespacial** con PostGIS (radio, bounding box)
- **Sincronización** con Google Places API
- **Gestión de fotos** y reviews
- **API REST** de alto rendimiento
- **Admin endpoints** para gestión de datos

---

## 🛠️ **Tecnologías**

- **Lenguaje:** Rust 1.83+
- **Framework:** Actix-web 4.4
- **Base de datos:** PostgreSQL 17 + PostGIS
- **ORM:** SQLx 0.7
- **Serialización:** Serde
- **Geolocalización:** PostGIS + geo-types

### **Dependencias Principales**

```toml
actix-web = "4.4"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
geo-types = "0.7"
geojson = "0.24"
```

---

## ✅ **Requisitos Previos**

### **Opción 1: Docker**

- Docker >= 24.0
- Docker Compose >= 2.20

### **Opción 2: Local**

- Rust 1.83+
- PostgreSQL 17+ con extensión PostGIS
- Cargo (viene con Rust)

---

## 📦 **Instalación**

### **Opción 1: Con Docker (Recomendado)**

Ver [README principal](../README.md) para instrucciones de Docker Compose.

### **Opción 2: Desarrollo Local**

```bash
# Instalar Rust (si no lo tienes)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Navegar al directorio
cd auphere-places

# Build del proyecto
cargo build --release

# O para desarrollo (sin optimizaciones)
cargo build
```

---

## ⚙️ **Configuración**

### **Variables de Entorno**

Crea un archivo `.env` en `auphere-places/`:

```env
# ============================================
# Database Configuration
# ============================================
DATABASE_URL=postgresql://auphere:password@localhost:5432/places

# ============================================
# Server Configuration
# ============================================
SERVER_ADDRESS=0.0.0.0
SERVER_PORT=8002
ENVIRONMENT=development
LOG_LEVEL=info

# ============================================
# Google Places API
# ============================================
GOOGLE_PLACES_API_KEY=your_google_places_api_key

# ============================================
# Admin Authentication
# ============================================
ADMIN_TOKEN=dev-admin-token

# ============================================
# Database Pool Configuration
# ============================================
DB_MAX_CONNECTIONS=20
DB_CONNECTION_TIMEOUT=30
```

### **Tabla de Variables**

| Variable                | Descripción                   | Requerido | Valor por Defecto                                    |
| ----------------------- | ----------------------------- | --------- | ---------------------------------------------------- |
| `DATABASE_URL`          | URL de PostgreSQL con PostGIS | ✅        | `postgresql://auphere:auphere@localhost:5432/places` |
| `SERVER_ADDRESS`        | Host del servidor             | ✅        | `0.0.0.0`                                            |
| `SERVER_PORT`           | Puerto del servidor           | ✅        | `8002`                                               |
| `ENVIRONMENT`           | Entorno de ejecución          | ✅        | `development`                                        |
| `LOG_LEVEL`             | Nivel de logging              | ✅        | `info`                                               |
| `GOOGLE_PLACES_API_KEY` | API Key de Google Places      | ⚠️        | -                                                    |
| `ADMIN_TOKEN`           | Token para endpoints de admin | ✅        | -                                                    |
| `DB_MAX_CONNECTIONS`    | Max conexiones en el pool     | ✅        | `20`                                                 |
| `DB_CONNECTION_TIMEOUT` | Timeout de conexión (seg)     | ✅        | `30`                                                 |

---

## 🏃 **Ejecución**

### **Desarrollo Local**

```bash
# Ejecutar en modo desarrollo (con hot reload usando cargo-watch)
cargo install cargo-watch
cargo watch -x run

# O ejecutar directamente
cargo run

# Build optimizado para producción
cargo build --release
./target/release/auphere-places
```

### **Con Docker**

```bash
# Desde la raíz del proyecto
docker-compose up places

# O build y run
docker build -t auphere-places .
docker run -p 8002:8002 --env-file .env auphere-places
```

### **Verificar que funciona**

```bash
# Health check
curl http://localhost:8002/health

# Debería responder:
# {"status":"ok","timestamp":"...","service":"auphere-places"}
```

---

## 🗄️ **Migraciones**

### **Ejecutar migraciones**

Las migraciones crean las tablas necesarias en PostgreSQL.

#### **Opción 1: Script automático**

```bash
# Desde auphere-places/
./run_migrations.sh
```

#### **Opción 2: Docker Compose**

```bash
# Desde la raíz del proyecto
for file in auphere-places/migrations/*.sql; do
  echo "Executing $(basename "$file")..."
  docker-compose exec -T postgres psql -U auphere -d places < "$file"
done
```

#### **Opción 3: Manualmente con psql**

```bash
psql -U auphere -d places < migrations/001_create_places.sql
psql -U auphere -d places < migrations/002_create_search_index.sql
psql -U auphere -d places < migrations/003_create_audit_tables.sql
psql -U auphere -d places < migrations/004_create_photos_table.sql
psql -U auphere -d places < migrations/005_adjust_google_rating_type.sql
psql -U auphere -d places < migrations/006_enrich_places_fields.sql
psql -U auphere -d places < migrations/007_fix_review_rating_type.sql
```

### **Verificar migraciones**

```bash
# Ver tablas creadas
psql -U auphere -d places -c "\dt"

# Debería mostrar:
# - places
# - photos
# - reviews
# - place_audit_log
# - search_queries
```

---

## 📚 **API Endpoints**

### **Places - Búsqueda**

| Método | Endpoint               | Descripción                    |
| ------ | ---------------------- | ------------------------------ |
| GET    | `/places/search`       | Buscar lugares con filtros     |
| GET    | `/places/{place_id}`   | Obtener detalle de lugar       |
| GET    | `/places/nearby`       | Lugares cercanos a coordenadas |
| GET    | `/places/autocomplete` | Autocompletar búsqueda         |

#### **Ejemplo: Búsqueda con filtros**

```bash
curl "http://localhost:8002/places/search?city=Zaragoza&category=restaurant&radius_km=5&lat=41.65&lon=-0.88"
```

**Query Parameters:**

- `city` - Ciudad (opcional)
- `category` - Categoría (opcional)
- `lat`, `lon` - Coordenadas (opcional)
- `radius_km` - Radio de búsqueda (opcional, default: 5)
- `page` - Página (default: 1)
- `limit` - Resultados por página (default: 20, max: 100)

### **Places - Admin**

| Método | Endpoint                   | Descripción                   |
| ------ | -------------------------- | ----------------------------- |
| POST   | `/admin/places`            | Crear lugar                   |
| PUT    | `/admin/places/{place_id}` | Actualizar lugar              |
| DELETE | `/admin/places/{place_id}` | Eliminar lugar                |
| POST   | `/admin/sync`              | Sincronizar con Google Places |

**⚠️ Requiere header:** `Authorization: Bearer {ADMIN_TOKEN}`

#### **Ejemplo: Sincronización**

```bash
curl -X POST http://localhost:8002/admin/sync \
  -H "Authorization: Bearer dev-admin-token" \
  -H "Content-Type: application/json" \
  -d '{
    "city": "Zaragoza",
    "country": "ES",
    "categories": ["restaurant", "cafe", "bar"]
  }'
```

### **Photos**

| Método | Endpoint                          | Descripción               |
| ------ | --------------------------------- | ------------------------- |
| GET    | `/places/{place_id}/photos`       | Obtener fotos de un lugar |
| POST   | `/admin/places/{place_id}/photos` | Añadir foto               |

### **Health & Metrics**

| Método | Endpoint   | Descripción                   |
| ------ | ---------- | ----------------------------- |
| GET    | `/health`  | Health check                  |
| GET    | `/metrics` | Métricas del servicio (admin) |

---

## 🧪 **Testing**

```bash
# Ejecutar tests unitarios
cargo test

# Con output detallado
cargo test -- --nocapture

# Test específico
cargo test test_search_places

# Con coverage (requiere tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### **Estructura de Tests**

```
auphere-places/
├── src/
│   ├── handlers/
│   │   └── places.rs    # Tests integrados
│   └── db/
│       └── repository.rs # Tests de DB
└── tests/
    └── integration_tests.rs
```

---

## 🐳 **Docker**

### **Build**

```bash
docker build -t auphere-places:latest .
```

La imagen usa **multi-stage build**:

- **Stage 1:** Compila el binario de Rust (grande, lento)
- **Stage 2:** Imagen runtime mínima con Debian slim (~50 MB)

### **Run**

```bash
docker run -p 8002:8002 \
  -e DATABASE_URL=postgresql://user:pass@postgres:5432/places \
  -e ADMIN_TOKEN=your-token \
  -e GOOGLE_PLACES_API_KEY=your-key \
  auphere-places:latest
```

---

## 📊 **Schema de Base de Datos**

### **Tabla: places**

| Campo             | Tipo             | Descripción            |
| ----------------- | ---------------- | ---------------------- |
| `id`              | UUID             | ID único               |
| `google_place_id` | VARCHAR          | ID de Google Places    |
| `name`            | VARCHAR          | Nombre del lugar       |
| `location`        | GEOGRAPHY(POINT) | Coordenadas (PostGIS)  |
| `city`            | VARCHAR          | Ciudad                 |
| `address`         | TEXT             | Dirección completa     |
| `category`        | VARCHAR          | Categoría principal    |
| `subcategories`   | JSONB            | Array de subcategorías |
| `rating`          | DECIMAL          | Rating promedio (0-5)  |
| `price_level`     | INTEGER          | Nivel de precio (1-4)  |
| `is_active`       | BOOLEAN          | Activo/Inactivo        |
| `created_at`      | TIMESTAMP        | Fecha de creación      |
| `updated_at`      | TIMESTAMP        | Última actualización   |

### **Índices**

- `idx_places_location_gist` - Índice geoespacial (GiST)
- `idx_places_city` - Búsqueda por ciudad
- `idx_places_category` - Búsqueda por categoría
- `idx_places_rating` - Ordenamiento por rating

---

## 🔧 **Troubleshooting**

### **Error: relation "places" does not exist**

```bash
# Las migraciones no se han ejecutado
# Ejecutar migraciones (ver sección Migraciones)
for file in auphere-places/migrations/*.sql; do
  docker-compose exec -T postgres psql -U auphere -d places < "$file"
done
```

### **Error: Connection refused (port 5432)**

```bash
# Verificar que PostgreSQL está corriendo
docker-compose ps postgres

# O si es local
pg_isready -U auphere -d places
```

### **Error: PostGIS extension not found**

```bash
# Instalar PostGIS en PostgreSQL
psql -U auphere -d places -c "CREATE EXTENSION IF NOT EXISTS postgis;"

# Verificar
psql -U auphere -d places -c "SELECT PostGIS_version();"
```

### **Error: cargo build failed**

```bash
# Verificar versión de Rust
rustc --version  # Debe ser 1.83+

# Actualizar Rust
rustup update

# Limpiar y rebuildar
cargo clean
cargo build
```

### **Error: Database pool connection timeout**

```bash
# Aumentar DB_MAX_CONNECTIONS y DB_CONNECTION_TIMEOUT
export DB_MAX_CONNECTIONS=50
export DB_CONNECTION_TIMEOUT=60
```

---

## 📁 **Estructura del Proyecto**

```
auphere-places/
├── src/
│   ├── main.rs              # Entry point
│   ├── config/              # Configuración
│   │   ├── db.rs
│   │   ├── env.rs
│   │   └── mod.rs
│   ├── db/                  # Capa de datos
│   │   ├── repository.rs    # Queries principales
│   │   ├── photo_repository.rs
│   │   └── mod.rs
│   ├── handlers/            # HTTP handlers
│   │   ├── places.rs
│   │   ├── admin.rs
│   │   ├── health.rs
│   │   └── mod.rs
│   ├── models/              # Structs y tipos
│   │   ├── place.rs
│   │   ├── photo.rs
│   │   └── mod.rs
│   ├── services/            # Lógica de negocio
│   │   ├── place_service.rs
│   │   ├── google_places_client.rs
│   │   └── mod.rs
│   └── errors.rs            # Error handling
├── migrations/              # SQL migrations
│   ├── 001_create_places.sql
│   ├── 002_create_search_index.sql
│   └── ...
├── Cargo.toml               # Dependencias
├── Dockerfile
└── README.md
```

---

## 🚀 **Performance**

### **Benchmarks**

- **Búsqueda simple:** ~1-3 ms
- **Búsqueda geoespacial:** ~5-10 ms
- **Insert:** ~2-5 ms
- **Throughput:** >10,000 requests/segundo (en hardware moderno)

### **Optimizaciones**

1. **Índices GiST** para búsquedas geoespaciales
2. **Connection pooling** con SQLx
3. **Async runtime** con Tokio
4. **Binary compilado** de Rust (sin VM/GC)

---

## 🔗 **Enlaces Útiles**

- [Actix-web Documentation](https://actix.rs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [PostGIS Documentation](https://postgis.net/docs/)
- [Rust Book](https://doc.rust-lang.org/book/)

---

## 📝 **Notas de Desarrollo**

### **Agregar nuevos endpoints**

1. Define el handler en `src/handlers/`
2. Registra la ruta en `src/main.rs`
3. Añade tests en el módulo correspondiente

### **Modificar schema**

1. Crea una nueva migración en `migrations/`
2. Ejecuta la migración
3. Actualiza los modelos en `src/models/`

### **Hot reload**

```bash
cargo install cargo-watch
cargo watch -x run
```

---

## 🤝 **Contribuir**

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request
