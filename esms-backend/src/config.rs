//! Configuration management module
//!
//! Loads and validates environment-based configuration.
//! Designed to be production-ready and easily extensible.

use serde::Deserialize;
use std::env;
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Invalid number format in environment variable")]
    ParseError,
}

/// Server configuration settings
#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

/// Sensor configuration settings
#[derive(Debug, Clone, Deserialize)]
pub struct SensorSettings {
    /// Interval in milliseconds between sensor readings
    pub interval_ms: u64,
}

/// FHIR configuration settings
#[derive(Debug, Clone, Deserialize)]
pub struct FhirSettings {
    /// Base URL for FHIR resources
    pub base_url: String,
    /// Patient reference for observations
    pub patient_reference: String,
}

/// Security configuration settings (JWT / TLS placeholders)
#[derive(Debug, Clone, Deserialize)]
pub struct SecuritySettings {
    pub jwt_secret: Option<String>,
    pub tls_enabled: bool,
}

/// Root configuration structure
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub sensor: SensorSettings,
    pub fhir: FhirSettings,
    pub security: SecuritySettings,
}

impl Settings {
    /// Load settings from environment variables
    pub fn from_env() -> Result<Self, SettingsError> {
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .map_err(|_| SettingsError::ParseError)?;

        let interval_ms = env::var("SENSOR_INTERVAL_MS")
            .unwrap_or_else(|_| "1000".into())
            .parse()
            .map_err(|_| SettingsError::ParseError)?;

        let tls_enabled = env::var("TLS_ENABLED")
            .unwrap_or_else(|_| "false".into())
            .parse()
            .map_err(|_| SettingsError::ParseError)?;

        Ok(Self {
            server: ServerSettings {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
                port,
            },
            sensor: SensorSettings {
                interval_ms,
            },
            fhir: FhirSettings {
                base_url: env::var("FHIR_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:8080/api/fhir".into()),
                patient_reference: env::var("FHIR_PATIENT_REFERENCE")
                    .unwrap_or_else(|_| "Patient/esms-monitor-subject".into()),
            },
            security: SecuritySettings {
                jwt_secret: env::var("JWT_SECRET").ok(),
                tls_enabled,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("SENSOR_INTERVAL_MS");

        let settings = Settings::from_env().unwrap();

        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.sensor.interval_ms, 1000);
    }

    #[test]
    fn test_custom_settings() {
        env::set_var("SERVER_PORT", "3000");
        env::set_var("SENSOR_INTERVAL_MS", "500");

        let settings = Settings::from_env().unwrap();

        assert_eq!(settings.server.port, 3000);
        assert_eq!(settings.sensor.interval_ms, 500);

        env::remove_var("SERVER_PORT");
        env::remove_var("SENSOR_INTERVAL_MS");
    }
}
