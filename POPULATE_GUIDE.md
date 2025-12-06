# 🗺️ Guía para Poblar la Base de Datos de Zaragoza

Este script (`populate_zaragoza.py`) automatiza el proceso de poblar la base de datos con lugares de Zaragoza usando Google Places API.

## 📋 Prerequisitos

### 1. Microservicio Running

El microservicio `auphere-places` debe estar corriendo:

```bash
cd auphere-places
cargo run
# O con auto-reload:
cargo watch -x run
```

### 2. Base de Datos PostgreSQL 17+

La base de datos debe estar creada y las migraciones ejecutadas:

```bash
# Crear base de datos
createdb places

# Ejecutar migraciones
cd auphere-places
./run_migrations.sh
```

### 3. Variables de Entorno

Tu archivo `.env` debe contener:

```env
# Base de datos
DATABASE_URL=postgresql://usuario:password@localhost:5432/places

# Google Places API (REQUERIDO para sincronización)
GOOGLE_PLACES_API_KEY=tu_api_key_aqui

# Token de administración (REQUERIDO)
ADMIN_TOKEN=tu_token_secreto_aqui

# Configuración del servidor
SERVER_ADDRESS=127.0.0.1
SERVER_PORT=3001
```

**Cómo obtener Google Places API Key:**

1. Ve a [Google Cloud Console](https://console.cloud.google.com/)
2. Crea o selecciona un proyecto
3. Habilita la API "Places API (New)"
4. Ve a "Credentials" y crea una API Key
5. Copia la API Key a tu `.env`

### 4. Instalar Dependencias Python

```bash
cd auphere-places
pip install -r requirements-populate.txt
```

## 🚀 Uso

### Ejecución Básica

```bash
cd auphere-places
python populate_zaragoza.py
```

### Lo que hace el script

El script sincronizará automáticamente los siguientes tipos de lugares en Zaragoza:

| Tipo | Icono | Nombre | Grid (km) | Radio (m) | Resultados Estimados |
|------|-------|--------|-----------|-----------|---------------------|
| `restaurant` | 🍽️ | Restaurantes | 1.5 | 1000 | ~450-500 |
| `bar` | 🍺 | Bares | 1.5 | 1000 | ~150-200 |
| `cafe` | ☕ | Cafeterías | 1.5 | 1000 | ~50-80 |
| `museum` | 🏛️ | Museos | 2.0 | 1500 | ~10-20 |
| `park` | 🌳 | Parques | 2.0 | 1500 | ~20-30 |
| `shopping_mall` | 🛍️ | Centros Comerciales | 2.5 | 2000 | ~5-10 |
| `lodging` | 🏨 | Hoteles | 2.0 | 1500 | ~30-50 |

### Salida del Script

El script mostrará:

```
🗺️  Poblador de Base de Datos - Zaragoza

Este script sincronizará los siguientes tipos de lugares desde Google Places API:
  🍽️ Restaurantes (grid: 1.5km, radio: 1000m)
  🍺 Bares (grid: 1.5km, radio: 1000m)
  ☕ Cafeterías (grid: 1.5km, radio: 1000m)
  ...

🔍 Verificando estado del servicio...
✅ Servicio disponible

📊 Estadísticas iniciales:
  restaurant: 0
  bar: 0
  ...

🚀 Iniciando sincronización...

✅ 🍽️ Restaurantes: 458 nuevos, 12 duplicados (125.3s)
✅ 🍺 Bares: 167 nuevos, 8 duplicados (98.2s)
...

📊 Estadísticas finales:
  restaurant: 458
  bar: 167
  ...

📋 Resumen de Sincronización
┌─────────────────────┬────────┬────────┬─────────────┬──────────┬──────────┐
│ Tipo                │ Estado │ Nuevos │ Duplicados  │ Requests │ Duración │
├─────────────────────┼────────┼────────┼─────────────┼──────────┼──────────┤
│ 🍽️ Restaurantes     │   ✅   │   458  │      12     │    55    │  125.3s  │
│ 🍺 Bares            │   ✅   │   167  │       8     │    48    │   98.2s  │
...
└─────────────────────┴────────┴────────┴─────────────┴──────────┴──────────┘

✨ Sincronización Completada

📊 Totales:
  • Lugares nuevos creados: 715
  • Lugares duplicados (omitidos): 45
  • Requests a Google Places API: 387
  • Duración total: 892.1s (14.9 min)

💰 Costo estimado: $6.58 USD
   (basado en $0.017 por request a Google Places API)
```

## 📊 Tiempos y Costos Estimados

### Por Primera Vez (Base de Datos Vacía)

- **Tiempo total**: 15-20 minutos
- **Lugares creados**: 700-800 lugares
- **Requests a Google API**: 350-400 requests
- **Costo estimado**: $6-7 USD

### Sincronizaciones Posteriores

Si ejecutas el script de nuevo:

- **Tiempo total**: 10-15 minutos (más rápido por deduplicación)
- **Lugares nuevos**: 50-100 (solo lugares que no existían antes)
- **Lugares duplicados**: La mayoría serán omitidos
- **Costo estimado**: $6-7 USD (mismo número de requests, pero menos inserts)

## 🔍 Verificación

### Ver lugares en la base de datos

```bash
# Búsqueda general
curl "http://localhost:3001/places/search?city=Zaragoza&limit=10"

# Búsqueda por tipo
curl "http://localhost:3001/places/search?city=Zaragoza&type=restaurant&limit=5"

# Búsqueda por texto
curl "http://localhost:3001/places/search?q=tapas&city=Zaragoza"

# Búsqueda geográfica (cerca del Pilar de Zaragoza)
curl "http://localhost:3001/places/search?lat=41.6561&lon=-0.8773&radius_km=2"
```

### Ver estadísticas

```bash
curl http://localhost:3001/admin/stats \
  -H "X-Admin-Token: tu_token_aqui"
```

### Consulta directa a PostgreSQL

```bash
# Contar lugares por tipo
psql places -c "SELECT type, COUNT(*) FROM places WHERE city = 'Zaragoza' GROUP BY type;"

# Ver lugares con mejor rating
psql places -c "SELECT name, type, google_rating FROM places WHERE city = 'Zaragoza' ORDER BY google_rating DESC LIMIT 10;"

# Lugares agregados recientemente
psql places -c "SELECT name, type, created_at FROM places WHERE city = 'Zaragoza' ORDER BY created_at DESC LIMIT 10;"
```

## 🔄 Actualizaciones Periódicas

### Cuándo Re-ejecutar el Script

Recomendamos ejecutar el script:

- **Mensualmente**: Para capturar nuevos lugares
- **Trimestralmente**: Para actualizaciones menos frecuentes
- **Después de eventos**: Si hay nuevas aperturas conocidas

### Deduplicación Automática

El script usa `google_place_id` para evitar duplicados:

- ✅ Lugares existentes → Se omiten automáticamente
- ✅ Lugares nuevos → Se agregan a la base de datos
- ⚠️ Datos actualizados (rating, horarios) → NO se actualizan automáticamente\*

\*Si necesitas actualizar datos existentes, tendrás que modificar el código del microservicio Rust.

## 🛠️ Troubleshooting

### Error: "El servicio no está disponible"

**Solución:**

```bash
# Terminal 1: Iniciar el microservicio
cd auphere-places
cargo run

# Terminal 2: Ejecutar el script
python populate_zaragoza.py
```

### Error: "ADMIN_TOKEN no está configurado"

**Solución:**

1. Edita tu archivo `.env`
2. Agrega: `ADMIN_TOKEN=mi-token-secreto-123` (usa un token seguro)
3. Reinicia el microservicio y ejecuta el script de nuevo

### Error: "GOOGLE_PLACES_API_KEY no está configurado"

**Solución:**

1. Obtén una API Key de Google Cloud Console (ver arriba)
2. Edita tu archivo `.env`
3. Agrega: `GOOGLE_PLACES_API_KEY=tu_api_key_aqui`
4. Reinicia el microservicio y ejecuta el script de nuevo

### Error: "Request denied" o "Invalid API Key"

**Posibles causas:**

1. API Key inválida o expirada
2. API "Places API (New)" no habilitada en Google Cloud
3. Restricciones de IP/dominio en la API Key
4. Cuota excedida

**Solución:**

1. Ve a Google Cloud Console
2. Verifica que "Places API (New)" esté habilitada
3. Verifica que la API Key tenga permisos
4. Revisa los límites de cuota

### El script encuentra pocos lugares (< 100)

**Posibles causas:**

1. Límites de cuota de Google Places API
2. Tipos de lugares mal configurados
3. Grid muy grande (celdas muy grandes, menos cobertura)

**Solución:**

1. Verifica tu cuota en Google Cloud Console
2. Ajusta los parámetros del grid en el script si es necesario

### Base de datos con duplicados

El microservicio usa `google_place_id` como UNIQUE constraint, así que esto no debería pasar. Si ves duplicados:

```bash
# Verificar duplicados
psql places -c "SELECT google_place_id, COUNT(*) FROM places GROUP BY google_place_id HAVING COUNT(*) > 1;"
```

Si hay duplicados (no debería), contacta con el equipo de desarrollo.

## 🎯 Personalización

### Modificar Tipos de Lugares

Edita el diccionario `PLACE_TYPES` en `populate_zaragoza.py`:

```python
PLACE_TYPES = {
    "restaurant": {
        "name_es": "Restaurantes",
        "icon": "🍽️",
        "cell_size_km": 1.5,  # Ajustar tamaño de grid
        "radius_m": 1000,      # Ajustar radio de búsqueda
    },
    # Agregar más tipos...
}
```

**Tipos de Google Places soportados:**

- `restaurant`, `bar`, `cafe`, `night_club`
- `museum`, `art_gallery`, `park`, `zoo`
- `shopping_mall`, `store`, `supermarket`
- `lodging`, `hotel`, `hostel`
- `movie_theater`, `gym`, `library`
- Y muchos más: [Lista completa](https://developers.google.com/maps/documentation/places/web-service/supported_types)

### Sincronizar Solo Algunos Tipos

Modifica la función `main()` en el script:

```python
# Sincronizar solo restaurantes y bares
manager.run_full_sync(place_types=["restaurant", "bar"])
```

### Ajustar Timeouts

Si tienes una conexión lenta:

```python
manager = PlacesSyncManager(
    base_url=base_url,
    admin_token=admin_token,
    timeout=600  # 10 minutos (default: 300s)
)
```

## 📚 Recursos Adicionales

- **Documentación del microservicio**: `README.md`
- **Guía de funcionamiento**: `GUIA_FUNCIONAMIENTO.md`
- **Quickstart**: `QUICKSTART.md`
- **Google Places API**: [Documentación oficial](https://developers.google.com/maps/documentation/places/web-service/overview)

## 💡 Consejos

1. **Primera ejecución**: Ejecuta el script fuera de horarios pico para no afectar la cuota de Google API
2. **Monitoreo**: Observa los logs del microservicio mientras el script corre
3. **Backups**: Haz un backup de la base de datos antes de ejecutar el script
4. **Pruebas**: Después de poblar, prueba búsquedas variadas para verificar calidad de datos

## 🎉 Siguiente Paso

Una vez que hayas poblado la base de datos:

1. ✅ Verifica que los datos estén en PostgreSQL
2. ✅ Prueba búsquedas desde el frontend
3. ✅ Configura tu agente para usar `search_places_tool`
4. ✅ Disfruta de búsquedas instantáneas sin límites

---

**¿Preguntas?** Revisa `GUIA_FUNCIONAMIENTO.md` o contacta al equipo de desarrollo.

