//! Environmental Stress Monitoring System (ESMS)
//!
//! Real-time backend for monitoring environmental and physiological
//! factors correlated with stress and discomfort.
//!
//! ⚠️ DISCLAIMER:
//! This system is NOT a medical diagnostic tool.

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod error;
mod fake_sensor;
mod fhir;
mod handlers;
mod models;
mod state;
mod validation;
mod websocket;

use crate::config::Settings;
use crate::fake_sensor::FakeSensorGenerator;
use crate::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Logging
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,esms=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json())
        .init();

    // Load configuration
    let settings = Settings::from_env().expect("Failed to load configuration");
    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);

    info!("Starting ESMS backend");
    info!("Binding server to {}", bind_address);

    // Shared application state
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // ---------------------------------------------------------------------
    // Fake sensor background task
    // IMPORTANT: use actix_rt::spawn (NOT tokio::spawn)
    // ---------------------------------------------------------------------
    let sensor_state = app_state.clone();
    let sensor_interval_ms = settings.sensor.interval_ms;

    actix_rt::spawn(async move {
        let generator = FakeSensorGenerator::new(sensor_interval_ms);
        generator.run(sensor_state).await;
    });

    // ---------------------------------------------------------------------
    // HTTP + WebSocket server
    // ---------------------------------------------------------------------
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(handlers::configure_routes)
    })
    .bind(&bind_address)?
    .run()
    .await
}
