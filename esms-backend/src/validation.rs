//! Input validation module
//! 
//! Provides comprehensive validation for sensor data and API inputs.

use crate::error::{AppError, AppResult};
use crate::models::{SensorInput, SensorReading};
use tracing::{debug, warn};
use validator::Validate;

/// Sensor data validation constraints
pub struct SensorConstraints;

impl SensorConstraints {
    /// DHT11 temperature range (Celsius)
    pub const TEMP_MIN: f64 = -40.0;
    pub const TEMP_MAX: f64 = 80.0;
    
    /// DHT11 humidity range (percentage)
    pub const HUMIDITY_MIN: f64 = 0.0;
    pub const HUMIDITY_MAX: f64 = 100.0;
    
    /// Sound sensor range (analog value)
    pub const SOUND_MIN: f64 = 0.0;
    pub const SOUND_MAX: f64 = 1023.0;
    
    /// MAX30100 heart rate range (BPM)
    pub const HR_MIN: f64 = 30.0;
    pub const HR_MAX: f64 = 220.0;
}

/// Validate sensor input data
pub fn validate_sensor_input(input: &SensorInput) -> AppResult<()> {
    // First, run struct-level validation
    if let Err(validation_errors) = input.validate() {
        let error_messages: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let msgs: Vec<&str> = errors
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|c| c.as_ref()))
                    .collect();
                format!("{}: {}", field, msgs.join(", "))
            })
            .collect();

        warn!(errors = ?error_messages, "Sensor input validation failed");
        return Err(AppError::ValidationError(error_messages.join("; ")));
    }

    // Additional semantic validation
    validate_temperature(input.temperature)?;
    validate_humidity(input.humidity)?;
    validate_sound(input.sound)?;
    validate_heart_rate(input.heart_rate)?;

    debug!("Sensor input validation passed");
    Ok(())
}

/// Validate sensor reading (internal data)
pub fn validate_sensor_reading(reading: &SensorReading) -> AppResult<()> {
    if let Err(validation_errors) = reading.validate() {
        let error_messages: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let msgs: Vec<&str> = errors
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|c| c.as_ref()))
                    .collect();
                format!("{}: {}", field, msgs.join(", "))
            })
            .collect();

        return Err(AppError::ValidationError(error_messages.join("; ")));
    }

    Ok(())
}

/// Validate temperature value
fn validate_temperature(value: f64) -> AppResult<()> {
    if !value.is_finite() {
        return Err(AppError::ValidationError(
            "Temperature must be a finite number".to_string(),
        ));
    }

    if value < SensorConstraints::TEMP_MIN || value > SensorConstraints::TEMP_MAX {
        return Err(AppError::ValidationError(format!(
            "Temperature {} out of valid range [{}, {}]",
            value,
            SensorConstraints::TEMP_MIN,
            SensorConstraints::TEMP_MAX
        )));
    }

    Ok(())
}

/// Validate humidity value
fn validate_humidity(value: f64) -> AppResult<()> {
    if !value.is_finite() {
        return Err(AppError::ValidationError(
            "Humidity must be a finite number".to_string(),
        ));
    }

    if value < SensorConstraints::HUMIDITY_MIN || value > SensorConstraints::HUMIDITY_MAX {
        return Err(AppError::ValidationError(format!(
            "Humidity {} out of valid range [{}, {}]",
            value,
            SensorConstraints::HUMIDITY_MIN,
            SensorConstraints::HUMIDITY_MAX
        )));
    }

    Ok(())
}

/// Validate sound level value
fn validate_sound(value: f64) -> AppResult<()> {
    if !value.is_finite() {
        return Err(AppError::ValidationError(
            "Sound level must be a finite number".to_string(),
        ));
    }

    if value < SensorConstraints::SOUND_MIN || value > SensorConstraints::SOUND_MAX {
        return Err(AppError::ValidationError(format!(
            "Sound level {} out of valid range [{}, {}]",
            value,
            SensorConstraints::SOUND_MIN,
            SensorConstraints::SOUND_MAX
        )));
    }

    Ok(())
}

/// Validate heart rate value
fn validate_heart_rate(value: f64) -> AppResult<()> {
    if !value.is_finite() {
        return Err(AppError::ValidationError(
            "Heart rate must be a finite number".to_string(),
        ));
    }

    if value < SensorConstraints::HR_MIN || value > SensorConstraints::HR_MAX {
        return Err(AppError::ValidationError(format!(
            "Heart rate {} out of physiologically valid range [{}, {}]",
            value,
            SensorConstraints::HR_MIN,
            SensorConstraints::HR_MAX
        )));
    }

    Ok(())
}

/// Validate pagination parameters
pub fn validate_pagination(page: Option<u32>, limit: Option<u32>) -> AppResult<(u32, u32)> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(100);

    if page == 0 {
        return Err(AppError::ValidationError(
            "Page number must be greater than 0".to_string(),
        ));
    }

    if limit == 0 || limit > 1000 {
        return Err(AppError::ValidationError(
            "Limit must be between 1 and 1000".to_string(),
        ));
    }

    Ok((page, limit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sensor_input() {
        let input = SensorInput {
            temperature: 25.0,
            humidity: 55.0,
            sound: 300.0,
            heart_rate: 72.0,
            timestamp: None,
        };

        assert!(validate_sensor_input(&input).is_ok());
    }

    #[test]
    fn test_invalid_temperature() {
        let input = SensorInput {
            temperature: 100.0, // Invalid: too high
            humidity: 55.0,
            sound: 300.0,
            heart_rate: 72.0,
            timestamp: None,
        };

        let result = validate_sensor_input(&input);
        assert!(result.is_err());
        
        if let Err(AppError::ValidationError(msg)) = result {
            assert!(msg.contains("temperature") || msg.contains("Temperature"));
        }
    }

    #[test]
    fn test_invalid_humidity() {
        let input = SensorInput {
            temperature: 25.0,
            humidity: 150.0, // Invalid: > 100%
            sound: 300.0,
            heart_rate: 72.0,
            timestamp: None,
        };

        assert!(validate_sensor_input(&input).is_err());
    }

    #[test]
    fn test_invalid_heart_rate_low() {
        let input = SensorInput {
            temperature: 25.0,
            humidity: 55.0,
            sound: 300.0,
            heart_rate: 20.0, // Invalid: too low
            timestamp: None,
        };

        assert!(validate_sensor_input(&input).is_err());
    }

    #[test]
    fn test_invalid_sound_level() {
        let input = SensorInput {
            temperature: 25.0,
            humidity: 55.0,
            sound: 2000.0, // Invalid: too high
            heart_rate: 72.0,
            timestamp: None,
        };

        assert!(validate_sensor_input(&input).is_err());
    }

    #[test]
    fn test_non_finite_values() {
        assert!(validate_temperature(f64::NAN).is_err());
        assert!(validate_temperature(f64::INFINITY).is_err());
        assert!(validate_humidity(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn test_pagination_validation() {
        // Valid cases
        assert!(validate_pagination(Some(1), Some(50)).is_ok());
        assert!(validate_pagination(None, None).is_ok());

        // Invalid cases
        assert!(validate_pagination(Some(0), Some(50)).is_err());
        assert!(validate_pagination(Some(1), Some(0)).is_err());
        assert!(validate_pagination(Some(1), Some(2000)).is_err());
    }
}
