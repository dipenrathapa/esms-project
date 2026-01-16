//! HTTP request handlers
//! 
//! Implements REST API endpoints for the ESMS application.

use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::fhir::{self, ObservationType};
use crate::models::{HealthCheck, SensorInput, SensorReading, WsMessage};
use crate::state::{AppState, ReadingStatistics};
use crate::validation::{validate_pagination, validate_sensor_input};
use crate::websocket::WsSession;

/// Configure all application routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Health check
            .route("/health", web::get().to(health_check))
            // Sensor data endpoints
            .route("/sensor/ingest", web::post().to(ingest_sensor_data))
            .route("/sensor/latest", web::get().to(get_latest_reading))
            .route("/sensor/history", web::get().to(get_reading_history))
            .route("/sensor/statistics", web::get().to(get_statistics))
            // FHIR endpoints
            .route("/fhir/Observation/latest", web::get().to(get_fhir_latest))
            .route("/fhir/Observation/bundle", web::get().to(get_fhir_bundle))
            .route(
                "/fhir/Observation/{type}/latest",
                web::get().to(get_fhir_observation_by_type),
            ),
    )
    // WebSocket endpoint
    .route("/ws", web::get().to(websocket_handler));
}

/// Health check endpoint
/// 
/// GET /api/health
/// 
/// Returns system health status including uptime and last reading time.
pub async fn health_check(
    state: web::Data<Arc<RwLock<AppState>>>,
) -> Result<HttpResponse, AppError> {
    let state = state.read().await;
    
    let health = HealthCheck {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: state.uptime_seconds(),
        last_reading: state.last_reading_time(),
    };

    Ok(HttpResponse::Ok().json(health))
}

/// Ingest sensor data
/// 
/// POST /api/sensor/ingest
/// 
/// Accepts sensor readings from external sources (for future hardware integration).
pub async fn ingest_sensor_data(
    state: web::Data<Arc<RwLock<AppState>>>,
    body: web::Json<SensorInput>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let correlation_id = extract_correlation_id(&req);
    
    info!(
        correlation_id = %correlation_id,
        "Received sensor data ingestion request"
    );

    // Validate input
    validate_sensor_input(&body)?;

    // Convert to reading
    let mut reading: SensorReading = body.into_inner().into();
    reading.correlation_id = Some(correlation_id.clone());

    // Store reading
    {
        let mut state = state.write().await;
        state.add_reading(reading.clone());
    }

    info!(
        correlation_id = %correlation_id,
        reading_id = %reading.id,
        "Sensor data ingested successfully"
    );

    Ok(HttpResponse::Created().json(IngestResponse {
        success: true,
        reading_id: reading.id.to_string(),
        correlation_id,
    }))
}

#[derive(Serialize)]
struct IngestResponse {
    success: bool,
    reading_id: String,
    correlation_id: String,
}

/// Get latest sensor reading
/// 
/// GET /api/sensor/latest
pub async fn get_latest_reading(
    state: web::Data<Arc<RwLock<AppState>>>,
) -> Result<HttpResponse, AppError> {
    let state = state.read().await;
    
    match state.get_latest() {
        Some(reading) => Ok(HttpResponse::Ok().json(reading)),
        None => Err(AppError::NotFound("No sensor readings available".to_string())),
    }
}

/// Query parameters for reading history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub minutes: Option<i64>,
}

/// Get reading history
/// 
/// GET /api/sensor/history?page=1&limit=100&minutes=60
pub async fn get_reading_history(
    state: web::Data<Arc<RwLock<AppState>>>,
    query: web::Query<HistoryQuery>,
) -> Result<HttpResponse, AppError> {
    let (page, limit) = validate_pagination(query.page, query.limit)?;
    let minutes = query.minutes.unwrap_or(60);

    let state = state.read().await;
    let readings: Vec<&SensorReading> = state.get_last_minutes(minutes);

    // Apply pagination
    let total = readings.len();
    let start = ((page - 1) * limit) as usize;
    let end = (start + limit as usize).min(total);

    let paginated: Vec<_> = if start < total {
        readings[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: paginated,
        page,
        limit,
        total: total as u32,
        total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
    }))
}

#[derive(Serialize)]
struct PaginatedResponse<T> {
    data: Vec<T>,
    page: u32,
    limit: u32,
    total: u32,
    total_pages: u32,
}

/// Get reading statistics
/// 
/// GET /api/sensor/statistics
pub async fn get_statistics(
    state: web::Data<Arc<RwLock<AppState>>>,
) -> Result<HttpResponse, AppError> {
    let state = state.read().await;
    let stats = state.get_statistics();

    Ok(HttpResponse::Ok().json(stats))
}

/// Get latest reading as FHIR Bundle
/// 
/// GET /api/fhir/Observation/latest
pub async fn get_fhir_latest(
    state: web::Data<Arc<RwLock<AppState>>>,
) -> Result<HttpResponse, AppError> {
    let state = state.read().await;
    
    let reading = state
        .get_latest()
        .ok_or_else(|| AppError::NotFound("No sensor readings available".to_string()))?;

    // Get patient reference from env or use default
    let patient_ref = std::env::var("FHIR_PATIENT_REFERENCE")
        .unwrap_or_else(|_| "Patient/esms-monitor-subject".to_string());

    let bundle = fhir::to_fhir_bundle(reading, &patient_ref)?;

    Ok(HttpResponse::Ok()
        .content_type("application/fhir+json")
        .json(bundle))
}

/// Get FHIR Bundle of recent observations
/// 
/// GET /api/fhir/Observation/bundle?count=10
#[derive(Debug, Deserialize)]
pub struct BundleQuery {
    pub count: Option<usize>,
}

pub async fn get_fhir_bundle(
    state: web::Data<Arc<RwLock<AppState>>>,
    query: web::Query<BundleQuery>,
) -> Result<HttpResponse, AppError> {
    let state = state.read().await;
    let count = query.count.unwrap_or(10).min(100);
    
    let readings = state.get_recent(count);
    if readings.is_empty() {
        return Err(AppError::NotFound("No sensor readings available".to_string()));
    }

    let patient_ref = std::env::var("FHIR_PATIENT_REFERENCE")
        .unwrap_or_else(|_| "Patient/esms-monitor-subject".to_string());

    // Create entries for all readings
    let mut all_entries = Vec::new();
    for reading in readings {
        let bundle = fhir::to_fhir_bundle(reading, &patient_ref)?;
        all_entries.extend(bundle.entry);
    }

    let combined_bundle = fhir::FhirBundle {
        resource_type: "Bundle".to_string(),
        id: Uuid::new_v4().to_string(),
        bundle_type: "collection".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        total: all_entries.len() as u32,
        entry: all_entries,
    };

    Ok(HttpResponse::Ok()
        .content_type("application/fhir+json")
        .json(combined_bundle))
}

/// Get specific observation type as FHIR
/// 
/// GET /api/fhir/Observation/{type}/latest
/// where type is: temperature, humidity, sound, heartrate
pub async fn get_fhir_observation_by_type(
    state: web::Data<Arc<RwLock<AppState>>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let obs_type_str = path.into_inner().to_lowercase();
    
    let obs_type = match obs_type_str.as_str() {
        "temperature" => ObservationType::Temperature,
        "humidity" => ObservationType::Humidity,
        "sound" | "soundlevel" => ObservationType::SoundLevel,
        "heartrate" | "heart_rate" | "hr" => ObservationType::HeartRate,
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid observation type: {}. Valid types: temperature, humidity, sound, heartrate",
                obs_type_str
            )))
        }
    };

    let state = state.read().await;
    let reading = state
        .get_latest()
        .ok_or_else(|| AppError::NotFound("No sensor readings available".to_string()))?;

    let patient_ref = std::env::var("FHIR_PATIENT_REFERENCE")
        .unwrap_or_else(|_| "Patient/esms-monitor-subject".to_string());

    let observation = fhir::to_fhir_observation(reading, obs_type, &patient_ref)?;

    Ok(HttpResponse::Ok()
        .content_type("application/fhir+json")
        .json(observation))
}

/// WebSocket upgrade handler
/// 
/// GET /ws
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<RwLock<AppState>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let client_id = Uuid::new_v4().to_string();
    
    info!(client_id = %client_id, "WebSocket connection request");
    
    // Register client
    {
        let mut state = state.write().await;
        state.add_client(client_id.clone());
    }

    let ws_session = WsSession::new(client_id, state.get_ref().clone());
    
    actix_web_actors::ws::start(ws_session, &req, stream)
}

/// Extract or generate correlation ID from request headers
fn extract_correlation_id(req: &HttpRequest) -> String {
    req.headers()
        .get("X-Correlation-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_ingest_valid_data() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_routes),
        )
        .await;

        let sensor_data = SensorInput {
            temperature: 25.0,
            humidity: 55.0,
            sound: 300.0,
            heart_rate: 72.0,
            timestamp: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/sensor/ingest")
            .set_json(&sensor_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    #[actix_web::test]
    async fn test_ingest_invalid_data() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_routes),
        )
        .await;

        let invalid_data = SensorInput {
            temperature: 100.0, // Invalid
            humidity: 55.0,
            sound: 300.0,
            heart_rate: 72.0,
            timestamp: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/sensor/ingest")
            .set_json(&invalid_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_get_latest_no_data() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/sensor/latest")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_fhir_observation_type() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        // Add a reading first
        {
            let mut s = state.write().await;
            s.add_reading(SensorReading::new(25.0, 55.0, 300.0, 72.0));
        }

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/fhir/Observation/temperature/latest")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
