//! Data models for sensor readings and related structures
//! 
//! Defines the core data structures used throughout the application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Raw sensor reading from the DHT11, Sound Level, and MAX30100 sensors
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SensorReading {
    /// Unique identifier for this reading
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Temperature reading from DHT11 sensor (Celsius)
    /// Valid range: -40°C to 80°C (DHT11 practical range: 0-50°C)
    #[validate(range(min = -40.0, max = 80.0, message = "Temperature must be between -40 and 80°C"))]
    pub temperature: f64,

    /// Humidity reading from DHT11 sensor (percentage)
    /// Valid range: 0% to 100% (DHT11 practical range: 20-90%)
    #[validate(range(min = 0.0, max = 100.0, message = "Humidity must be between 0 and 100%"))]
    pub humidity: f64,

    /// Sound level reading (arbitrary units, typically 0-1023 for analog sensors)
    /// Higher values indicate louder environments
    #[validate(range(min = 0.0, max = 1023.0, message = "Sound level must be between 0 and 1023"))]
    pub sound: f64,

    /// Heart rate reading from MAX30100 sensor (beats per minute)
    /// Valid physiological range: 30-220 BPM
    #[validate(range(min = 30.0, max = 220.0, message = "Heart rate must be between 30 and 220 BPM"))]
    pub heart_rate: f64,

    /// ISO 8601 timestamp of the reading
    pub timestamp: DateTime<Utc>,

    /// Optional correlation ID for request tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl SensorReading {
    /// Create a new sensor reading with current timestamp
    pub fn new(temperature: f64, humidity: f64, sound: f64, heart_rate: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            temperature,
            humidity,
            sound,
            heart_rate,
            timestamp: Utc::now(),
            correlation_id: Some(Uuid::new_v4().to_string()),
        }
    }

    /// Check if environmental conditions indicate potential stress factors
    /// This is NOT a diagnostic tool - only identifies correlations
    pub fn stress_indicators(&self) -> StressIndicators {
        StressIndicators {
            high_temperature: self.temperature > 28.0,
            low_temperature: self.temperature < 18.0,
            high_humidity: self.humidity > 70.0,
            low_humidity: self.humidity < 30.0,
            high_noise: self.sound > 500.0,
            elevated_heart_rate: self.heart_rate > 100.0,
            low_heart_rate: self.heart_rate < 50.0,
        }
    }
}

/// Stress indicator flags (non-diagnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressIndicators {
    pub high_temperature: bool,
    pub low_temperature: bool,
    pub high_humidity: bool,
    pub low_humidity: bool,
    pub high_noise: bool,
    pub elevated_heart_rate: bool,
    pub low_heart_rate: bool,
}

impl StressIndicators {
    /// Count the number of active stress indicators
    pub fn active_count(&self) -> u8 {
        let mut count = 0;
        if self.high_temperature { count += 1; }
        if self.low_temperature { count += 1; }
        if self.high_humidity { count += 1; }
        if self.low_humidity { count += 1; }
        if self.high_noise { count += 1; }
        if self.elevated_heart_rate { count += 1; }
        if self.low_heart_rate { count += 1; }
        count
    }
}

/// Input DTO for sensor data ingestion
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SensorInput {
    #[validate(range(min = -40.0, max = 80.0))]
    pub temperature: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub humidity: f64,

    #[validate(range(min = 0.0, max = 1023.0))]
    pub sound: f64,

    #[validate(range(min = 30.0, max = 220.0))]
    pub heart_rate: f64,

    /// Optional client-provided timestamp (defaults to server time)
    pub timestamp: Option<DateTime<Utc>>,
}

impl From<SensorInput> for SensorReading {
    fn from(input: SensorInput) -> Self {
        SensorReading {
            id: Uuid::new_v4(),
            temperature: input.temperature,
            humidity: input.humidity,
            sound: input.sound,
            heart_rate: input.heart_rate,
            timestamp: input.timestamp.unwrap_or_else(Utc::now),
            correlation_id: Some(Uuid::new_v4().to_string()),
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// New sensor reading available
    SensorUpdate(SensorReading),
    /// Connection acknowledgment
    Connected { client_id: String },
    /// Error message
    Error { message: String },
    /// Heartbeat/ping
    Ping,
    /// Heartbeat/pong response
    Pong,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub last_reading: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_reading_creation() {
        let reading = SensorReading::new(25.0, 55.0, 300.0, 72.0);
        
        assert_eq!(reading.temperature, 25.0);
        assert_eq!(reading.humidity, 55.0);
        assert_eq!(reading.sound, 300.0);
        assert_eq!(reading.heart_rate, 72.0);
        assert!(reading.correlation_id.is_some());
    }

    #[test]
    fn test_stress_indicators_normal() {
        let reading = SensorReading::new(22.0, 50.0, 200.0, 70.0);
        let indicators = reading.stress_indicators();
        
        assert!(!indicators.high_temperature);
        assert!(!indicators.high_humidity);
        assert!(!indicators.high_noise);
        assert!(!indicators.elevated_heart_rate);
        assert_eq!(indicators.active_count(), 0);
    }

    #[test]
    fn test_stress_indicators_high_temp() {
        let reading = SensorReading::new(32.0, 50.0, 200.0, 70.0);
        let indicators = reading.stress_indicators();
        
        assert!(indicators.high_temperature);
        assert_eq!(indicators.active_count(), 1);
    }

    #[test]
    fn test_stress_indicators_multiple() {
        let reading = SensorReading::new(32.0, 80.0, 600.0, 110.0);
        let indicators = reading.stress_indicators();
        
        assert!(indicators.high_temperature);
        assert!(indicators.high_humidity);
        assert!(indicators.high_noise);
        assert!(indicators.elevated_heart_rate);
        assert_eq!(indicators.active_count(), 4);
    }

    #[test]
    fn test_sensor_input_conversion() {
        let input = SensorInput {
            temperature: 24.0,
            humidity: 60.0,
            sound: 250.0,
            heart_rate: 75.0,
            timestamp: None,
        };
        
        let reading: SensorReading = input.into();
        
        assert_eq!(reading.temperature, 24.0);
        assert_eq!(reading.humidity, 60.0);
    }

    #[test]
    fn test_sensor_reading_validation() {
        use validator::Validate;
        
        // Valid reading
        let valid = SensorReading::new(25.0, 55.0, 300.0, 72.0);
        assert!(valid.validate().is_ok());
        
        // Invalid temperature (too high)
        let mut invalid = SensorReading::new(100.0, 55.0, 300.0, 72.0);
        assert!(invalid.validate().is_err());
        
        // Invalid heart rate (too low)
        invalid = SensorReading::new(25.0, 55.0, 300.0, 20.0);
        assert!(invalid.validate().is_err());
    }
}
