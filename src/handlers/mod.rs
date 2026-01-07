// src/handlers/mod.rs
// DOCUMENTATION: Handlers module organization
// PURPOSE: Re-export handler components

pub mod health;
pub mod places;

pub use health::config as health_config;
pub use places::config as places_config;
