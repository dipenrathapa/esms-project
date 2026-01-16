//! Application state management
//! 
//! Central state container for the ESMS application, managing sensor readings
//! and WebSocket client connections.

use chrono::{DateTime, Duration, Utc};
use std::collections::VecDeque;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::SensorReading;

/// Maximum number of readings to keep in memory
const MAX_READINGS: usize = 3600; // 1 hour at 1 reading/second

/// Central application state
#[derive(Debug)]
pub struct AppState {
    /// Circular buffer of sensor readings
    readings: VecDeque<SensorReading>,
    /// Application start time
    start_time: DateTime<Utc>,
    /// Total readings processed
    total_readings: u64,
    /// Connected WebSocket clients
    connected_clients: Vec<String>,
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        info!("Initializing application state");
        Self {
            readings: VecDeque::with_capacity(MAX_READINGS),
            start_time: Utc::now(),
            total_readings: 0,
            connected_clients: Vec::new(),
        }
    }

    /// Add a new sensor reading to the buffer
    pub fn add_reading(&mut self, reading: SensorReading) {
        self.total_readings += 1;

        // Remove oldest reading if at capacity
        if self.readings.len() >= MAX_READINGS {
            self.readings.pop_front();
        }

        debug!(
            reading_id = %reading.id,
            total = self.total_readings,
            "Adding sensor reading to state"
        );

        self.readings.push_back(reading);
    }

    /// Get the latest sensor reading
    pub fn get_latest(&self) -> Option<&SensorReading> {
        self.readings.back()
    }

    /// Get the last N readings
    pub fn get_recent(&self, count: usize) -> Vec<&SensorReading> {
        self.readings.iter().rev().take(count).collect()
    }

    /// Get readings within a time range
    pub fn get_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&SensorReading> {
        self.readings
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect()
    }

    /// Get readings from the last N minutes
    pub fn get_last_minutes(&self, minutes: i64) -> Vec<&SensorReading> {
        let cutoff = Utc::now() - Duration::minutes(minutes);
        self.readings
            .iter()
            .filter(|r| r.timestamp >= cutoff)
            .collect()
    }

    /// Get all readings (for export/analysis)
    pub fn get_all(&self) -> Vec<&SensorReading> {
        self.readings.iter().collect()
    }

    /// Get statistics about the readings
    pub fn get_statistics(&self) -> ReadingStatistics {
        if self.readings.is_empty() {
            return ReadingStatistics::empty();
        }

        let (temp_sum, hum_sum, sound_sum, hr_sum) = self.readings.iter().fold(
            (0.0, 0.0, 0.0, 0.0),
            |(t, h, s, hr), r| {
                (
                    t + r.temperature,
                    h + r.humidity,
                    s + r.sound,
                    hr + r.heart_rate,
                )
            },
        );

        let count = self.readings.len() as f64;

        ReadingStatistics {
            count: self.readings.len(),
            avg_temperature: temp_sum / count,
            avg_humidity: hum_sum / count,
            avg_sound: sound_sum / count,
            avg_heart_rate: hr_sum / count,
            min_temperature: self.readings.iter().map(|r| r.temperature).fold(f64::MAX, f64::min),
            max_temperature: self.readings.iter().map(|r| r.temperature).fold(f64::MIN, f64::max),
            min_heart_rate: self.readings.iter().map(|r| r.heart_rate).fold(f64::MAX, f64::min),
            max_heart_rate: self.readings.iter().map(|r| r.heart_rate).fold(f64::MIN, f64::max),
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        (Utc::now() - self.start_time).num_seconds() as u64
    }

    /// Get total readings processed
    pub fn total_readings(&self) -> u64 {
        self.total_readings
    }

    /// Get the timestamp of the latest reading
    pub fn last_reading_time(&self) -> Option<DateTime<Utc>> {
        self.readings.back().map(|r| r.timestamp)
    }

    /// Register a new WebSocket client
    pub fn add_client(&mut self, client_id: String) {
        info!(client_id = %client_id, "WebSocket client connected");
        self.connected_clients.push(client_id);
    }

    /// Remove a WebSocket client
    pub fn remove_client(&mut self, client_id: &str) {
        info!(client_id = %client_id, "WebSocket client disconnected");
        self.connected_clients.retain(|id| id != client_id);
    }

    /// Get count of connected clients
    pub fn client_count(&self) -> usize {
        self.connected_clients.len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistical summary of readings
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingStatistics {
    pub count: usize,
    pub avg_temperature: f64,
    pub avg_humidity: f64,
    pub avg_sound: f64,
    pub avg_heart_rate: f64,
    pub min_temperature: f64,
    pub max_temperature: f64,
    pub min_heart_rate: f64,
    pub max_heart_rate: f64,
}

impl ReadingStatistics {
    pub fn empty() -> Self {
        Self {
            count: 0,
            avg_temperature: 0.0,
            avg_humidity: 0.0,
            avg_sound: 0.0,
            avg_heart_rate: 0.0,
            min_temperature: 0.0,
            max_temperature: 0.0,
            min_heart_rate: 0.0,
            max_heart_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.get_latest().is_none());
        assert_eq!(state.total_readings(), 0);
    }

    #[test]
    fn test_add_reading() {
        let mut state = AppState::new();
        let reading = SensorReading::new(25.0, 55.0, 300.0, 72.0);
        
        state.add_reading(reading.clone());
        
        assert_eq!(state.total_readings(), 1);
        let latest = state.get_latest().unwrap();
        assert_eq!(latest.temperature, 25.0);
    }

    #[test]
    fn test_circular_buffer() {
        let mut state = AppState::new();
        
        // Add MAX_READINGS + 10 readings
        for i in 0..(MAX_READINGS + 10) {
            let reading = SensorReading::new(
                20.0 + (i as f64 * 0.01),
                50.0,
                200.0,
                70.0,
            );
            state.add_reading(reading);
        }
        
        // Should only have MAX_READINGS stored
        assert_eq!(state.readings.len(), MAX_READINGS);
        // But total count should be all readings
        assert_eq!(state.total_readings(), (MAX_READINGS + 10) as u64);
    }

    #[test]
    fn test_get_recent() {
        let mut state = AppState::new();
        
        for i in 0..10 {
            state.add_reading(SensorReading::new(20.0 + i as f64, 50.0, 200.0, 70.0));
        }
        
        let recent = state.get_recent(5);
        assert_eq!(recent.len(), 5);
        // Most recent should be first
        assert_eq!(recent[0].temperature, 29.0);
    }

    #[test]
    fn test_statistics() {
        let mut state = AppState::new();
        
        state.add_reading(SensorReading::new(20.0, 40.0, 100.0, 60.0));
        state.add_reading(SensorReading::new(30.0, 60.0, 200.0, 80.0));
        
        let stats = state.get_statistics();
        
        assert_eq!(stats.count, 2);
        assert_eq!(stats.avg_temperature, 25.0);
        assert_eq!(stats.avg_humidity, 50.0);
        assert_eq!(stats.min_temperature, 20.0);
        assert_eq!(stats.max_temperature, 30.0);
    }

    #[test]
    fn test_client_management() {
        let mut state = AppState::new();
        
        state.add_client("client-1".to_string());
        state.add_client("client-2".to_string());
        
        assert_eq!(state.client_count(), 2);
        
        state.remove_client("client-1");
        
        assert_eq!(state.client_count(), 1);
    }
}
