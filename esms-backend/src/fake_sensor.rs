//! Fake Sensor Data Generator
//! 
//! ══════════════════════════════════════════════════════════════════════════════
//! SINGLE SOURCE OF SENSOR DATA
//! ══════════════════════════════════════════════════════════════════════════════
//! 
//! This file generates realistic fake sensor data for testing and development.
//! It is designed to be the SINGLE SOURCE of sensor data in the system.
//! 
//! When real hardware is connected (Arduino Uno with DHT11, Sound Sensor, and 
//! MAX30100), this file should be replaced with a serial communication module
//! without changing the rest of the system.
//! 
//! SENSORS SIMULATED:
//! 1. DHT11 - Temperature & Humidity sensor
//! 2. Sound Level Sensor - Ambient noise detection
//! 3. MAX30100 - Heart Rate sensor (pulse oximeter)
//! 
//! The generated data follows realistic patterns including:
//! - Circadian rhythm simulation for heart rate
//! - Environmental correlation between temperature and humidity
//! - Random noise events for sound sensor
//! - Gradual drift patterns to simulate real sensor behavior

use chrono::Utc;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::models::SensorReading;
use crate::state::AppState;

/// Fake sensor generator that produces realistic sensor data
pub struct FakeSensorGenerator {
    /// Interval between readings in milliseconds
    interval_ms: u64,
    /// Base temperature (simulated room temperature)
    base_temperature: f64,
    /// Base humidity (simulated ambient humidity)
    base_humidity: f64,
    /// Base sound level (simulated ambient noise)
    base_sound: f64,
    /// Base heart rate (resting heart rate)
    base_heart_rate: f64,
    /// Time drift factor for gradual changes
    drift_factor: f64,
}

impl FakeSensorGenerator {
    /// Create a new fake sensor generator
    pub fn new(interval_ms: u64) -> Self {
        info!(
            interval_ms = interval_ms,
            "Initializing fake sensor generator"
        );

        Self {
            interval_ms,
            base_temperature: 22.0,  // Comfortable room temperature
            base_humidity: 50.0,      // Normal indoor humidity
            base_sound: 150.0,        // Quiet room baseline
            base_heart_rate: 70.0,    // Normal resting heart rate
            drift_factor: 0.0,
        }
    }

    /// Run the fake sensor generator continuously
    pub async fn run(mut self, state: Arc<RwLock<AppState>>) {
        info!("Starting fake sensor data generation loop");

        let mut tick_interval = interval(Duration::from_millis(self.interval_ms));
        let mut rng = rand::thread_rng();

        // Initialize random distributions for realistic data
        let temp_noise = Normal::new(0.0, 0.5).unwrap();
        let humidity_noise = Normal::new(0.0, 2.0).unwrap();
        let sound_noise = Normal::new(0.0, 30.0).unwrap();
        let hr_noise = Normal::new(0.0, 3.0).unwrap();

        let mut tick_count: u64 = 0;

        loop {
            tick_interval.tick().await;
            tick_count += 1;

            // Update drift factor for gradual environmental changes
            self.drift_factor += 0.01;
            if self.drift_factor > std::f64::consts::PI * 2.0 {
                self.drift_factor = 0.0;
            }

            // Generate temperature with circadian and environmental simulation
            // DHT11 sensor characteristics: ±2°C accuracy, 0-50°C range
            let temp_drift = (self.drift_factor * 0.5).sin() * 3.0;
            let temp_variation = temp_noise.sample(&mut rng);
            let temperature = (self.base_temperature + temp_drift + temp_variation)
                .clamp(0.0, 50.0);

            // Generate humidity inversely correlated with temperature (realistic behavior)
            // DHT11 sensor characteristics: ±5% accuracy, 20-90% range
            let humidity_drift = -temp_drift * 2.0; // Inverse correlation
            let humidity_variation = humidity_noise.sample(&mut rng);
            let humidity = (self.base_humidity + humidity_drift + humidity_variation)
                .clamp(20.0, 90.0);

            // Generate sound level with random spike events
            // Simulates quiet baseline with occasional noise events
            let sound_base_variation = sound_noise.sample(&mut rng);
            let sound_spike = if rng.gen::<f64>() < 0.05 {
                // 5% chance of noise spike (door closing, conversation, etc.)
                rng.gen_range(200.0..500.0)
            } else {
                0.0
            };
            let sound = (self.base_sound + sound_base_variation + sound_spike)
                .clamp(0.0, 1023.0);

            // Generate heart rate with circadian rhythm simulation
            // MAX30100 sensor: optical heart rate detection
            // Simulates higher HR during "day" (activity) and lower during "rest"
            let hr_circadian = (self.drift_factor * 2.0).sin() * 10.0;
            let hr_variation = hr_noise.sample(&mut rng);
            let heart_rate = (self.base_heart_rate + hr_circadian + hr_variation)
                .clamp(50.0, 120.0);

            // Create sensor reading
            let reading = SensorReading {
                id: Uuid::new_v4(),
                temperature: (temperature * 10.0).round() / 10.0, // 1 decimal precision
                humidity: (humidity * 10.0).round() / 10.0,
                sound: sound.round(),
                heart_rate: heart_rate.round(),
                timestamp: Utc::now(),
                correlation_id: Some(Uuid::new_v4().to_string()),
            };

            debug!(
                tick = tick_count,
                temperature = reading.temperature,
                humidity = reading.humidity,
                sound = reading.sound,
                heart_rate = reading.heart_rate,
                "Generated fake sensor reading"
            );

            // Store reading in application state
            {
                let mut app_state = state.write().await;
                app_state.add_reading(reading.clone());
            }

            // Log stress indicators periodically
            if tick_count % 60 == 0 {
                let indicators = reading.stress_indicators();
                let active = indicators.active_count();
                if active > 0 {
                    warn!(
                        active_stress_indicators = active,
                        high_temp = indicators.high_temperature,
                        high_humidity = indicators.high_humidity,
                        high_noise = indicators.high_noise,
                        elevated_hr = indicators.elevated_heart_rate,
                        "Stress indicators detected (non-diagnostic monitoring)"
                    );
                }
            }

            // Occasionally simulate environmental changes
            if tick_count % 300 == 0 {
                // Every 5 minutes, slight baseline shift
                self.base_temperature += rng.gen_range(-1.0..1.0);
                self.base_temperature = self.base_temperature.clamp(18.0, 28.0);
                
                self.base_humidity += rng.gen_range(-5.0..5.0);
                self.base_humidity = self.base_humidity.clamp(35.0, 65.0);

                info!(
                    new_base_temp = self.base_temperature,
                    new_base_humidity = self.base_humidity,
                    "Environmental baseline shift simulated"
                );
            }
        }
    }
}

/// Represents expected serial data format from Arduino
/// When replacing fake data with real sensors, parse incoming serial data into this format
#[derive(Debug)]
#[allow(dead_code)]
pub struct ArduinoSerialFormat {
    /// Raw format expected from Arduino:
    /// "T:26.4,H:61.2,S:180,HR:78\n"
    /// 
    /// Parsing example:
    /// ```ignore
    /// fn parse_arduino_data(line: &str) -> Option<SensorReading> {
    ///     let parts: HashMap<&str, &str> = line
    ///         .split(',')
    ///         .filter_map(|p| p.split_once(':'))
    ///         .collect();
    ///     
    ///     Some(SensorReading::new(
    ///         parts.get("T")?.parse().ok()?,
    ///         parts.get("H")?.parse().ok()?,
    ///         parts.get("S")?.parse().ok()?,
    ///         parts.get("HR")?.parse().ok()?,
    ///     ))
    /// }
    /// ```
    _marker: (),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = FakeSensorGenerator::new(1000);
        assert_eq!(generator.interval_ms, 1000);
        assert_eq!(generator.base_temperature, 22.0);
        assert_eq!(generator.base_humidity, 50.0);
    }

    #[tokio::test]
    async fn test_single_reading_generation() {
        use std::time::Duration;
        use tokio::time::timeout;

        let state = Arc::new(RwLock::new(AppState::new()));
        let generator = FakeSensorGenerator::new(100);
        
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            generator.run(state_clone).await;
        });

        // Wait for at least one reading
        timeout(Duration::from_millis(500), async {
            loop {
                let readings = state.read().await;
                if readings.get_latest().is_some() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .expect("Timeout waiting for sensor reading");

        // Verify reading was generated
        let readings = state.read().await;
        let latest = readings.get_latest().unwrap();
        
        assert!(latest.temperature >= 0.0 && latest.temperature <= 50.0);
        assert!(latest.humidity >= 20.0 && latest.humidity <= 90.0);
        assert!(latest.sound >= 0.0 && latest.sound <= 1023.0);
        assert!(latest.heart_rate >= 50.0 && latest.heart_rate <= 120.0);

        handle.abort();
    }
}
