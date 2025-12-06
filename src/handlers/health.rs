// src/handlers/health.rs
// DOCUMENTATION: Health check handler
// PURPOSE: Simple endpoint to verify service status

use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "auphere-places",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check));
}
