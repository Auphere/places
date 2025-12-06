// src/services/grid_generator.rs
// DOCUMENTATION: Geographic grid generation for city coverage
// PURPOSE: Generate search grid cells to systematically cover a city area

use serde::{Deserialize, Serialize};

/// Represents a single grid cell for searching
/// DOCUMENTATION: Each cell represents a search area for Google Places API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    /// Center point latitude
    pub latitude: f64,
    /// Center point longitude
    pub longitude: f64,
    /// Search radius in meters
    pub radius: u32,
    /// Cell identifier (for debugging/logging)
    pub cell_id: String,
}

/// City boundary definition
/// DOCUMENTATION: Defines geographic boundaries for a city
#[derive(Debug, Clone)]
pub struct CityBounds {
    /// City name
    pub name: String,
    /// Minimum latitude (south)
    pub min_lat: f64,
    /// Maximum latitude (north)
    pub max_lat: f64,
    /// Minimum longitude (west)
    pub min_lng: f64,
    /// Maximum longitude (east)
    pub max_lng: f64,
}

/// Grid generator service
/// DOCUMENTATION: Generates systematic grid coverage for cities
pub struct GridGenerator;

impl GridGenerator {
    /// Generate grid cells for a city
    /// DOCUMENTATION: Creates a grid of search points to cover the entire city
    ///
    /// Uses a grid spacing that accounts for Earth's curvature:
    /// - Cell size: approximately 1.5km x 1.5km
    /// - Search radius: 1000m (to ensure overlap and complete coverage)
    ///
    /// # Arguments
    /// * `bounds` - City geographic boundaries
    /// * `cell_size_km` - Size of each grid cell in kilometers (default 1.5)
    /// * `radius_m` - Search radius for each cell in meters (default 1000)
    ///
    /// # Returns
    /// Vector of GridCell objects covering the city
    pub fn generate_grid(bounds: &CityBounds, cell_size_km: f64, radius_m: u32) -> Vec<GridCell> {
        let mut cells = Vec::new();

        // Calculate grid steps
        // 1 degree of latitude ≈ 111 km
        // 1 degree of longitude ≈ 111 km * cos(latitude)
        let lat_step = cell_size_km / 111.0;

        // Use center latitude for longitude calculation
        let center_lat = (bounds.min_lat + bounds.max_lat) / 2.0;
        let lng_step = cell_size_km / (111.0 * center_lat.to_radians().cos());

        log::info!(
            "Generating grid for {}: lat_step={:.4}°, lng_step={:.4}°",
            bounds.name,
            lat_step,
            lng_step
        );

        // Generate grid points
        let mut cell_counter = 0;
        let mut lat = bounds.min_lat;

        while lat <= bounds.max_lat {
            let mut lng = bounds.min_lng;

            while lng <= bounds.max_lng {
                cell_counter += 1;

                cells.push(GridCell {
                    latitude: lat,
                    longitude: lng,
                    radius: radius_m,
                    cell_id: format!("{}-{}", bounds.name, cell_counter),
                });

                lng += lng_step;
            }

            lat += lat_step;
        }

        log::info!(
            "Generated {} grid cells for {} (coverage: {:.2} km²)",
            cells.len(),
            bounds.name,
            Self::calculate_area_coverage(bounds)
        );

        cells
    }

    /// Get predefined city bounds
    /// DOCUMENTATION: Returns geographic boundaries for known cities
    ///
    /// # Arguments
    /// * `city_name` - Name of the city (case-insensitive)
    ///
    /// # Returns
    /// Option containing CityBounds if city is known
    pub fn get_city_bounds(city_name: &str) -> Option<CityBounds> {
        let city_lower = city_name.to_lowercase();

        match city_lower.as_str() {
            // ACTIVE CITY FOR TESTING
            "zaragoza" => Some(CityBounds {
                name: "Zaragoza".to_string(),
                min_lat: 41.6000, // South boundary
                max_lat: 41.7000, // North boundary
                min_lng: -0.9500, // West boundary
                max_lng: -0.8200, // East boundary
            }),

            // OTHER CITIES (commented out for testing phase)
            // Uncomment these when ready to expand to other cities
            /*
            "madrid" => Some(CityBounds {
                name: "Madrid".to_string(),
                min_lat: 40.3119,
                max_lat: 40.5615,
                min_lng: -3.8871,
                max_lng: -3.5179,
            }),
            "barcelona" => Some(CityBounds {
                name: "Barcelona".to_string(),
                min_lat: 41.3200,
                max_lat: 41.4695,
                min_lng: 2.0524,
                max_lng: 2.2280,
            }),
            "valencia" => Some(CityBounds {
                name: "Valencia".to_string(),
                min_lat: 39.4200,
                max_lat: 39.5200,
                min_lng: -0.4300,
                max_lng: -0.3000,
            }),
            "sevilla" | "seville" => Some(CityBounds {
                name: "Sevilla".to_string(),
                min_lat: 37.3200,
                max_lat: 37.4300,
                min_lng: -6.0500,
                max_lng: -5.9200,
            }),
            "bilbao" => Some(CityBounds {
                name: "Bilbao".to_string(),
                min_lat: 43.2300,
                max_lat: 43.2900,
                min_lng: -2.9800,
                max_lng: -2.9000,
            }),
            "malaga" | "málaga" => Some(CityBounds {
                name: "Málaga".to_string(),
                min_lat: 36.6800,
                max_lat: 36.7600,
                min_lng: -4.4800,
                max_lng: -4.3800,
            }),
            */
            _ => {
                log::warn!(
                    "Unknown city: {}. Only Zaragoza is configured for testing.",
                    city_name
                );
                None
            }
        }
    }

    /// Generate grid for a known city
    /// DOCUMENTATION: Convenience method to generate grid using predefined city bounds
    ///
    /// # Arguments
    /// * `city_name` - Name of the city
    /// * `cell_size_km` - Optional cell size (defaults to 1.5 km)
    /// * `radius_m` - Optional radius (defaults to 1000 m)
    ///
    /// # Returns
    /// Result containing grid cells or error if city is unknown
    pub fn generate_for_city(
        city_name: &str,
        cell_size_km: Option<f64>,
        radius_m: Option<u32>,
    ) -> Result<Vec<GridCell>, String> {
        let bounds = Self::get_city_bounds(city_name)
            .ok_or_else(|| format!("Unknown city: {}", city_name))?;

        let cells = Self::generate_grid(
            &bounds,
            cell_size_km.unwrap_or(1.5),
            radius_m.unwrap_or(1000),
        );

        Ok(cells)
    }

    /// Calculate approximate area coverage in km²
    /// DOCUMENTATION: Estimates the geographic area covered by bounds
    fn calculate_area_coverage(bounds: &CityBounds) -> f64 {
        let lat_diff = bounds.max_lat - bounds.min_lat;
        let lng_diff = bounds.max_lng - bounds.min_lng;

        // Approximate calculation (good enough for rough estimates)
        let center_lat = (bounds.min_lat + bounds.max_lat) / 2.0;
        let lat_km = lat_diff * 111.0;
        let lng_km = lng_diff * 111.0 * center_lat.to_radians().cos();

        lat_km * lng_km
    }

    /// Optimize grid by removing cells outside actual city boundaries
    /// DOCUMENTATION: Future enhancement - filter cells using actual city polygon
    /// Currently returns all cells (no filtering implemented)
    #[allow(dead_code)]
    pub fn optimize_grid(cells: Vec<GridCell>) -> Vec<GridCell> {
        // TODO: Implement polygon-based filtering
        // For now, return all cells
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_generation() {
        let bounds = CityBounds {
            name: "Test City".to_string(),
            min_lat: 40.0,
            max_lat: 40.1,
            min_lng: -3.7,
            max_lng: -3.6,
        };

        let cells = GridGenerator::generate_grid(&bounds, 1.5, 1000);

        // Should generate multiple cells
        assert!(cells.len() > 0);

        // All cells should have the specified radius
        assert!(cells.iter().all(|c| c.radius == 1000));

        // All cells should be within bounds (with some tolerance)
        for cell in &cells {
            assert!(cell.latitude >= bounds.min_lat - 0.01);
            assert!(cell.latitude <= bounds.max_lat + 0.01);
            assert!(cell.longitude >= bounds.min_lng - 0.01);
            assert!(cell.longitude <= bounds.max_lng + 0.01);
        }
    }

    #[test]
    fn test_city_bounds_madrid() {
        let bounds = GridGenerator::get_city_bounds("Madrid");
        assert!(bounds.is_some());

        let bounds = bounds.unwrap();
        assert_eq!(bounds.name, "Madrid");
        assert!(bounds.min_lat < bounds.max_lat);
        assert!(bounds.min_lng < bounds.max_lng);
    }

    #[test]
    fn test_city_bounds_case_insensitive() {
        let bounds1 = GridGenerator::get_city_bounds("Madrid");
        let bounds2 = GridGenerator::get_city_bounds("MADRID");
        let bounds3 = GridGenerator::get_city_bounds("madrid");

        assert!(bounds1.is_some());
        assert!(bounds2.is_some());
        assert!(bounds3.is_some());
    }

    #[test]
    fn test_generate_for_city() {
        let result = GridGenerator::generate_for_city("Barcelona", None, None);
        assert!(result.is_ok());

        let cells = result.unwrap();
        assert!(cells.len() > 0);
    }

    #[test]
    fn test_unknown_city() {
        let result = GridGenerator::generate_for_city("UnknownCity", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_area_calculation() {
        let bounds = CityBounds {
            name: "Test".to_string(),
            min_lat: 40.0,
            max_lat: 40.1,
            min_lng: -3.7,
            max_lng: -3.6,
        };

        let area = GridGenerator::calculate_area_coverage(&bounds);
        assert!(area > 0.0);
        // Should be roughly 100-150 km² for this size
        assert!(area > 50.0 && area < 200.0);
    }
}
