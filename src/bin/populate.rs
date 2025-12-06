// src/bin/populate.rs
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process;
use std::time::{Duration, Instant};

// --- Colores ANSI para la terminal ---
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";

// --- Estructuras de Datos ---

#[derive(Debug, Clone)]
struct PlaceTypeConfig {
    name_es: &'static str,
    icon: &'static str,
    cell_size_km: f64,
    radius_m: i32,
}

#[derive(Serialize)]
struct SyncPayload {
    place_type: String,
    cell_size_km: f64,
    radius_m: i32,
}

#[derive(Deserialize, Debug, Default)]
struct SyncResponse {
    city: String,
    #[serde(default)]
    api_requests: u32,
    #[serde(default)]
    places_retrieved: u32,
    #[serde(default)]
    places_created: u32,
    #[serde(default)]
    places_skipped: u32,
    #[serde(default)]
    places_failed: u32,
    #[serde(default)]
    reviews_created: u32,
    #[serde(default)]
    photos_created: u32,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default)]
    duration_seconds: u64,
    #[serde(default)]
    started_at: String,
    #[serde(default)]
    completed_at: Option<String>,
}

#[derive(Debug)]
struct SyncResult {
    place_type: String,
    type_name: String,
    icon: String,
    success: bool,
    places_created: u32,
    places_skipped: u32,
    api_requests: u32,
    duration_secs: f64,
}

// --- Configuración de Tipos de Lugares ---
// Implementado como una función que devuelve el HashMap para facilitar el manejo

fn get_place_types() -> HashMap<&'static str, PlaceTypeConfig> {
    let mut m = HashMap::new();

    // === OCIO Y ENTRETENIMIENTO (Tabla A) ===
    m.insert("amusement_center", PlaceTypeConfig { name_es: "Centros de Entretenimiento", icon: "🎮", cell_size_km: 2.5, radius_m: 1500 });
    m.insert("amusement_park", PlaceTypeConfig { name_es: "Parques de Atracciones", icon: "🎢", cell_size_km: 3.0, radius_m: 2000 });
    m.insert("aquarium", PlaceTypeConfig { name_es: "Acuarios", icon: "🐠", cell_size_km: 3.0, radius_m: 2000 });
    m.insert("banquet_hall", PlaceTypeConfig { name_es: "Salones de Banquetes", icon: "🎊", cell_size_km: 2.0, radius_m: 1500 });
    m.insert("bowling_alley", PlaceTypeConfig { name_es: "Boleras", icon: "🎳", cell_size_km: 2.5, radius_m: 1500 });
    m.insert("casino", PlaceTypeConfig { name_es: "Casinos", icon: "🎰", cell_size_km: 2.5, radius_m: 1500 });
    m.insert("hiking_area", PlaceTypeConfig { name_es: "Zonas de Senderismo", icon: "🥾", cell_size_km: 3.0, radius_m: 2000 });
    m.insert("historical_landmark", PlaceTypeConfig { name_es: "Monumentos Históricos", icon: "🏛️", cell_size_km: 2.0, radius_m: 1500 });
    m.insert("marina", PlaceTypeConfig { name_es: "Marinas", icon: "⛵", cell_size_km: 3.0, radius_m: 2000 });
    m.insert("movie_theater", PlaceTypeConfig { name_es: "Cines", icon: "🎬", cell_size_km: 2.5, radius_m: 1500 });
    m.insert("national_park", PlaceTypeConfig { name_es: "Parques Nacionales", icon: "🏞️", cell_size_km: 5.0, radius_m: 3000 });
    m.insert("night_club", PlaceTypeConfig { name_es: "Discotecas", icon: "💃", cell_size_km: 2.0, radius_m: 1000 });
    m.insert("park", PlaceTypeConfig { name_es: "Parques", icon: "🌳", cell_size_km: 2.0, radius_m: 1500 });
    m.insert("tourist_attraction", PlaceTypeConfig { name_es: "Atracciones Turísticas", icon: "📸", cell_size_km: 2.0, radius_m: 1500 });
    m.insert("visitor_center", PlaceTypeConfig { name_es: "Centros de Visitantes", icon: "ℹ️", cell_size_km: 2.5, radius_m: 1500 });
    m.insert("zoo", PlaceTypeConfig { name_es: "Zoológicos", icon: "🦁", cell_size_km: 3.0, radius_m: 2000 });

    // === COMIDAS Y BEBIDAS (Tabla A) ===
    m.insert("american_restaurant", PlaceTypeConfig { name_es: "Restaurantes Americanos", icon: "🍔", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("bakery", PlaceTypeConfig { name_es: "Panaderías", icon: "🥖", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("bar", PlaceTypeConfig { name_es: "Bares", icon: "🍺", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("barbecue_restaurant", PlaceTypeConfig { name_es: "Restaurantes de Barbacoa", icon: "🍖", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("brazilian_restaurant", PlaceTypeConfig { name_es: "Restaurantes Brasileños", icon: "🇧🇷", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("breakfast_restaurant", PlaceTypeConfig { name_es: "Restaurantes de Desayuno", icon: "🥞", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("brunch_restaurant", PlaceTypeConfig { name_es: "Restaurantes de Brunch", icon: "🥂", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("cafe", PlaceTypeConfig { name_es: "Cafeterías", icon: "☕", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("chinese_restaurant", PlaceTypeConfig { name_es: "Restaurantes Chinos", icon: "🥡", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("coffee_shop", PlaceTypeConfig { name_es: "Cafés", icon: "☕", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("fast_food_restaurant", PlaceTypeConfig { name_es: "Comida Rápida", icon: "🍟", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("french_restaurant", PlaceTypeConfig { name_es: "Restaurantes Franceses", icon: "🇫🇷", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("greek_restaurant", PlaceTypeConfig { name_es: "Restaurantes Griegos", icon: "🇬🇷", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("hamburger_restaurant", PlaceTypeConfig { name_es: "Hamburgueserías", icon: "🍔", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("ice_cream_shop", PlaceTypeConfig { name_es: "Heladerías", icon: "🍦", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("indian_restaurant", PlaceTypeConfig { name_es: "Restaurantes Indios", icon: "🇮🇳", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("indonesian_restaurant", PlaceTypeConfig { name_es: "Restaurantes Indonesios", icon: "🇮🇩", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("italian_restaurant", PlaceTypeConfig { name_es: "Restaurantes Italianos", icon: "🍝", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("japanese_restaurant", PlaceTypeConfig { name_es: "Restaurantes Japoneses", icon: "🍣", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("korean_restaurant", PlaceTypeConfig { name_es: "Restaurantes Coreanos", icon: "🇰🇷", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("lebanese_restaurant", PlaceTypeConfig { name_es: "Restaurantes Libaneses", icon: "🇱🇧", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("meal_delivery", PlaceTypeConfig { name_es: "Comida a Domicilio", icon: "🛵", cell_size_km: 2.0, radius_m: 1000 });
    m.insert("meal_takeaway", PlaceTypeConfig { name_es: "Comida para Llevar", icon: "🥡", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("mediterranean_restaurant", PlaceTypeConfig { name_es: "Restaurantes Mediterráneos", icon: "🫒", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("mexican_restaurant", PlaceTypeConfig { name_es: "Restaurantes Mexicanos", icon: "🌮", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("middle_eastern_restaurant", PlaceTypeConfig { name_es: "Restaurantes de Oriente Medio", icon: "🥙", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("pizza_restaurant", PlaceTypeConfig { name_es: "Pizzerías", icon: "🍕", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("ramen_restaurant", PlaceTypeConfig { name_es: "Restaurantes de Ramen", icon: "🍜", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("restaurant", PlaceTypeConfig { name_es: "Restaurantes", icon: "🍽️", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("sandwich_shop", PlaceTypeConfig { name_es: "Bocadillerías", icon: "🥪", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("seafood_restaurant", PlaceTypeConfig { name_es: "Restaurantes de Mariscos", icon: "🦐", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("spanish_restaurant", PlaceTypeConfig { name_es: "Restaurantes Españoles", icon: "🇪🇸", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("steak_house", PlaceTypeConfig { name_es: "Asadores", icon: "🥩", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("sushi_restaurant", PlaceTypeConfig { name_es: "Restaurantes de Sushi", icon: "🍱", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("thai_restaurant", PlaceTypeConfig { name_es: "Restaurantes Tailandeses", icon: "🇹🇭", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("turkish_restaurant", PlaceTypeConfig { name_es: "Restaurantes Turcos", icon: "🇹🇷", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("vegan_restaurant", PlaceTypeConfig { name_es: "Restaurantes Veganos", icon: "🥗", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("vegetarian_restaurant", PlaceTypeConfig { name_es: "Restaurantes Vegetarianos", icon: "🥕", cell_size_km: 1.5, radius_m: 1000 });
    m.insert("vietnamese_restaurant", PlaceTypeConfig { name_es: "Restaurantes Vietnamitas", icon: "🇻🇳", cell_size_km: 1.5, radius_m: 1000 });

    m
}

// --- Manager Logic ---

struct PlacesSyncManager {
    base_url: String,
    admin_token: String,
    client: Client,
    results: Vec<SyncResult>,
}

impl PlacesSyncManager {
    fn new(base_url: String, admin_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            admin_token,
            client,
            results: Vec::new(),
        }
    }

    async fn check_service_health(&self) -> bool {
        match self.client.get(format!("{}/health", self.base_url)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    async fn sync_place_type(&self, place_type: &str, config: &PlaceTypeConfig) -> Result<SyncResponse, String> {
        let url = format!("{}/admin/sync/Zaragoza", self.base_url);
        let payload = SyncPayload {
            place_type: place_type.to_string(),
            cell_size_km: config.cell_size_km,
            radius_m: config.radius_m,
        };

        let response = self
            .client
            .post(&url)
            .header("X-Admin-Token", &self.admin_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            response
                .json::<SyncResponse>()
                .await
                .map_err(|e| format!("Failed to parse response JSON: {}", e))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("HTTP {} - {}", status, body))
        }
    }

    async fn run_full_sync(&mut self) {
        println!("\n{}🔍 Checking service status...{}", CYAN, RESET);
        if !self.check_service_health().await {
            println!("{}❌ Service unavailable.{}", RED, RESET);
            println!("{}Please ensure auphere-places is running (cargo run){}", YELLOW, RESET);
            process::exit(1);
        }
        println!("{}✅ Service available{}\n", GREEN, RESET);

        let place_types = get_place_types();
        self.print_header(place_types.len());

        println!("\n{}🚀 Starting synchronization...{}\n", BOLD, RESET);

        // Sort keys to have deterministic order (alphabetical)
        let mut keys: Vec<&str> = place_types.keys().cloned().collect();
        keys.sort();

        let total_types = keys.len();

        for (i, key) in keys.iter().enumerate() {
            let config = place_types.get(key).unwrap();
            let start_time = Instant::now();

            println!(
                "{}[{}/{}] Syncing {} {}...{}", 
                CYAN, i + 1, total_types, config.icon, config.name_es, RESET
            );

            let response = self.sync_place_type(key, config).await;
            let duration = start_time.elapsed().as_secs_f64();

            match response {
                Ok(resp) => {
                    println!(
                        "{}✅ {} {}: {} new, {} skipped ({:.1}s){}",
                        GREEN, config.icon, config.name_es,
                        resp.places_created, resp.places_skipped, duration,
                        RESET
                    );
                    if !resp.errors.is_empty() {
                        println!(
                            "{}⚠️  {} warnings ({}).{}",
                            YELLOW,
                            config.name_es,
                            resp.errors.join("; "),
                            RESET
                        );
                    }
                    self.results.push(SyncResult {
                        place_type: key.to_string(),
                        type_name: config.name_es.to_string(),
                        icon: config.icon.to_string(),
                        success: true,
                        places_created: resp.places_created,
                        places_skipped: resp.places_skipped,
                        api_requests: resp.api_requests,
                        duration_secs: duration,
                    });
                }
                Err(err_msg) => {
                    println!(
                        "{}❌ Error syncing {}: {}{}",
                        RED, config.name_es, err_msg, RESET
                    );
                    self.results.push(SyncResult {
                        place_type: key.to_string(),
                        type_name: config.name_es.to_string(),
                        icon: config.icon.to_string(),
                        success: false,
                        places_created: 0,
                        places_skipped: 0,
                        api_requests: 0,
                        duration_secs: duration,
                    });
                }
            }

            // Small pause to be nice
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        self.print_summary();
    }

    fn print_header(&self, total_count: usize) {
        println!("{}╔══════════════════════════════════════════════════════════════╗{}", CYAN, RESET);
        println!("{}║   🗺️  Database Populator - Zaragoza (Places API)              ║{}", CYAN, RESET);
        println!("{}╚══════════════════════════════════════════════════════════════╝{}", CYAN, RESET);
        println!("\n{}📊 Total types to sync: {}{}", BOLD, total_count, RESET);
    }

    fn print_summary(&self) {
        println!("\n\n{}📋 Synchronization Summary{}", BOLD, RESET);
        println!("──────────────────────────────────────────────────────────────────────────────");
        println!(
            "{:<40} {:<10} {:>10} {:>10} {:>10}",
            "Type", "Status", "New", "Skipped", "Duration"
        );
        println!("──────────────────────────────────────────────────────────────────────────────");

        let mut total_created = 0;
        let mut total_skipped = 0;
        let mut total_requests = 0;
        let mut total_duration = 0.0;

        for res in &self.results {
            let status_icon = if res.success { "✅" } else { "❌" };
            println!(
                "{:<40} {:<10} {:>10} {:>10} {:>9.1}s",
                format!("{} {}", res.icon, res.type_name),
                status_icon,
                res.places_created,
                res.places_skipped,
                res.duration_secs
            );

            if res.success {
                total_created += res.places_created;
                total_skipped += res.places_skipped;
                total_requests += res.api_requests;
                total_duration += res.duration_secs;
            }
        }

        println!("──────────────────────────────────────────────────────────────────────────────");
        println!("\n{}✨ Process Completed Successfully{}", GREEN, RESET);
        println!("{}📊 Totals:{}", BOLD, RESET);
        println!("  • New places created: {}{}{}", GREEN, total_created, RESET);
        println!("  • Duplicates skipped: {}{}{}", YELLOW, total_skipped, RESET);
        println!("  • API Requests: {}{}{}", BLUE, total_requests, RESET);
        println!("  • Total Duration: {:.1}s ({:.1} min)", total_duration, total_duration / 60.0);
        
        let cost = total_requests as f64 * 0.017;
        println!("\n{}💰 Estimated Cost: ${:.2} USD{}", BOLD, cost, RESET);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let admin_token = env::var("ADMIN_TOKEN").expect("ADMIN_TOKEN must be set in .env");
    let base_url = env::var("PLACES_API_URL").unwrap_or_else(|_| "http://localhost:3001".to_string());
    
    // Check if GOOGLE_PLACES_API_KEY is present just to warn user (logic is in server, but good practice)
    if env::var("GOOGLE_PLACES_API_KEY").is_err() {
        println!("{}⚠️  GOOGLE_PLACES_API_KEY not found in .env. Server sync might fail if not configured there.{}", YELLOW, RESET);
    }

    let mut manager = PlacesSyncManager::new(base_url, admin_token);
    manager.run_full_sync().await;
}

