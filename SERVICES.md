# Auphere Places — Servicios y funcionalidad

Este documento describe los endpoints expuestos por `auphere-places`, su comportamiento y cómo se persisten los datos.

## Endpoints

### `GET /health`

Health check del servicio.

---

### `GET /places/search` (y alias `GET /places/nearby`)

Busca lugares.

- **Si `GOOGLE_PLACES_API_KEY` está configurada**: usa Google Places (nearby si hay `lat/lon`, o text search si no hay coords).
- **Si no hay key**: hace fallback a búsqueda en DB.

Query params principales:
- `q`: texto libre
- `city`: ciudad (útil para text search)
- `type`: tipo (restaurant/bar/cafe/…)
- `lat`, `lon`, `radius_km`: sesgo por proximidad
- `page`, `limit`

Respuesta: `FrontendSearchResponse` con `places[]`.

---

### `GET /places/{id}`

Retorna el **detalle** de un lugar.

`id` puede ser:
- UUID interno (si ya lo tienes en DB), o
- `google_place_id` (lo usual; es el ID universal para UI/agent).

Comportamiento:
- Si el lugar **no existe** en DB y hay `GOOGLE_PLACES_API_KEY`: se crea on-demand desde Google Place Details.
- Si el lugar existe pero está “stale” o sin assets: refresca desde Google Place Details (best-effort).
- Si existe `FOURSQUARE_API_KEY`: intenta match con Foursquare y persiste:
  - `photos` (source=`foursquare`)
  - `tips` en tabla dedicada `place_tips` (sin rating)
  - guarda `fsq_id` en `places.tags.external_ids.fsq_id`

Respuesta: `PlaceDetailResponse` con:
- campos del place (flatten)
- `photos[]` (multi-source)
- `reviews[]` (Google)
- `tips[]` (Foursquare)

---

### `GET /places/clusters`

Clustering por “zonas” usando **PostGIS DBSCAN** (DB-only). Útil para reducir tokens aguas arriba (agent) y armar itinerarios por zonas.

Query params:
- `city` (recomendado)
- `type`
- opcional: `lat`, `lon`, `radius_km`
- `eps_m` (default 800)
- `min_points` (default 3)
- `limit_places` (default 1000)
- `limit_clusters` (default 20)

Respuesta: `ClusterResponse` con:
- `clusters[]` (centroide + places mínimos)
- `unclustered[]`

## Persistencia (Postgres)

Tablas principales:
- `places`: registro canónico del lugar (PostGIS `location`, rating, price_level, opening_hours, etc.)
- `place_photos`: fotos multi-fuente (`source`, `source_photo_reference`, urls)
- `place_reviews`: reviews con rating (v1: Google)
- `place_tips`: tips/notes sin rating (v1: Foursquare)

## Foursquare (compatibilidad docs)

Para Foursquare Places API se envían headers requeridos:
- `Authorization: Bearer <FOURSQUARE_API_KEY>`
- `X-Places-Api-Version: 2025-06-17`

Endpoints utilizados:
- `/places/search`
- `/places/{fsq_id}/photos`
- `/places/{fsq_id}/tips`


