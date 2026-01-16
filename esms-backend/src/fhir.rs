//! FHIR R4 Compliance Module
//! 
//! Converts sensor readings to FHIR R4 Observation resources for healthcare interoperability.
//! 
//! LOINC Code Mappings:
//! - Temperature: 8310-5 (Body temperature)
//! - Humidity: Custom environmental code (no standard LOINC)
//! - Sound Level: Custom acoustic stress code (no standard LOINC)
//! - Heart Rate: 8867-4 (Heart rate)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::SensorReading;

/// LOINC codes for sensor observations
pub mod loinc {
    /// Body temperature - LOINC 8310-5
    pub const TEMPERATURE: &str = "8310-5";
    pub const TEMPERATURE_DISPLAY: &str = "Body temperature";
    
    /// Heart rate - LOINC 8867-4
    pub const HEART_RATE: &str = "8867-4";
    pub const HEART_RATE_DISPLAY: &str = "Heart rate";
    
    /// Custom code for environmental humidity (no standard LOINC)
    pub const HUMIDITY: &str = "ESMS-ENV-001";
    pub const HUMIDITY_DISPLAY: &str = "Environmental humidity";
    
    /// Custom code for ambient sound level (no standard LOINC)
    pub const SOUND_LEVEL: &str = "ESMS-ENV-002";
    pub const SOUND_LEVEL_DISPLAY: &str = "Ambient sound level";
}

/// FHIR coding system URLs
pub mod systems {
    pub const LOINC: &str = "http://loinc.org";
    pub const ESMS_CUSTOM: &str = "http://esms.local/fhir/CodeSystem/environmental";
    pub const UCUM: &str = "http://unitsofmeasure.org";
}

/// FHIR R4 Observation resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirObservation {
    /// Resource type (always "Observation")
    pub resource_type: String,
    
    /// Logical ID of the resource
    pub id: String,
    
    /// Metadata about the resource
    pub meta: FhirMeta,
    
    /// Observation status
    pub status: String,
    
    /// Category of the observation
    pub category: Vec<FhirCodeableConcept>,
    
    /// Code describing what was observed
    pub code: FhirCodeableConcept,
    
    /// Subject of the observation (patient reference)
    pub subject: FhirReference,
    
    /// Time of the observation
    pub effective_date_time: String,
    
    /// Time the observation was recorded
    pub issued: String,
    
    /// The observed value
    pub value_quantity: FhirQuantity,
    
    /// Interpretation of the result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<Vec<FhirCodeableConcept>>,
    
    /// Reference range for the observation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_range: Option<Vec<FhirReferenceRange>>,
    
    /// Device that made the observation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<FhirReference>,
    
    /// Additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Vec<FhirAnnotation>>,
}

/// FHIR resource metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirMeta {
    pub version_id: String,
    pub last_updated: String,
    pub profile: Vec<String>,
}

/// FHIR codeable concept
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirCodeableConcept {
    pub coding: Vec<FhirCoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// FHIR coding element
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirCoding {
    pub system: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// FHIR reference to another resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirReference {
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// FHIR quantity value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirQuantity {
    pub value: f64,
    pub unit: String,
    pub system: String,
    pub code: String,
}

/// FHIR reference range
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirReferenceRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<FhirQuantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<FhirQuantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// FHIR annotation (note)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirAnnotation {
    pub text: String,
    pub time: String,
}

/// Type of observation being converted
#[derive(Debug, Clone, Copy)]
pub enum ObservationType {
    Temperature,
    Humidity,
    SoundLevel,
    HeartRate,
}

impl ObservationType {
    fn loinc_code(&self) -> &'static str {
        match self {
            ObservationType::Temperature => loinc::TEMPERATURE,
            ObservationType::Humidity => loinc::HUMIDITY,
            ObservationType::SoundLevel => loinc::SOUND_LEVEL,
            ObservationType::HeartRate => loinc::HEART_RATE,
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            ObservationType::Temperature => loinc::TEMPERATURE_DISPLAY,
            ObservationType::Humidity => loinc::HUMIDITY_DISPLAY,
            ObservationType::SoundLevel => loinc::SOUND_LEVEL_DISPLAY,
            ObservationType::HeartRate => loinc::HEART_RATE_DISPLAY,
        }
    }

    fn coding_system(&self) -> &'static str {
        match self {
            ObservationType::Temperature | ObservationType::HeartRate => systems::LOINC,
            ObservationType::Humidity | ObservationType::SoundLevel => systems::ESMS_CUSTOM,
        }
    }

    fn unit(&self) -> (&'static str, &'static str) {
        match self {
            ObservationType::Temperature => ("Cel", "°C"),
            ObservationType::Humidity => ("%", "%"),
            ObservationType::SoundLevel => ("1", "units"),
            ObservationType::HeartRate => ("/min", "beats/minute"),
        }
    }

    fn category(&self) -> &'static str {
        match self {
            ObservationType::Temperature | ObservationType::HeartRate => "vital-signs",
            ObservationType::Humidity | ObservationType::SoundLevel => "survey",
        }
    }
}

/// Convert a sensor reading to a FHIR Observation
pub fn to_fhir_observation(
    reading: &SensorReading,
    obs_type: ObservationType,
    patient_reference: &str,
) -> AppResult<FhirObservation> {
    let value = match obs_type {
        ObservationType::Temperature => reading.temperature,
        ObservationType::Humidity => reading.humidity,
        ObservationType::SoundLevel => reading.sound,
        ObservationType::HeartRate => reading.heart_rate,
    };

    let (unit_code, unit_display) = obs_type.unit();
    let observation_id = format!("{}_{:?}", reading.id, obs_type).to_lowercase();

    Ok(FhirObservation {
        resource_type: "Observation".to_string(),
        id: observation_id,
        meta: FhirMeta {
            version_id: "1".to_string(),
            last_updated: Utc::now().to_rfc3339(),
            profile: vec![
                "http://hl7.org/fhir/StructureDefinition/Observation".to_string(),
            ],
        },
        status: "final".to_string(),
        category: vec![FhirCodeableConcept {
            coding: vec![FhirCoding {
                system: "http://terminology.hl7.org/CodeSystem/observation-category".to_string(),
                code: obs_type.category().to_string(),
                display: Some(obs_type.category().to_string()),
            }],
            text: None,
        }],
        code: FhirCodeableConcept {
            coding: vec![FhirCoding {
                system: obs_type.coding_system().to_string(),
                code: obs_type.loinc_code().to_string(),
                display: Some(obs_type.display_name().to_string()),
            }],
            text: Some(obs_type.display_name().to_string()),
        },
        subject: FhirReference {
            reference: patient_reference.to_string(),
            display: Some("ESMS Monitoring Subject".to_string()),
        },
        effective_date_time: reading.timestamp.to_rfc3339(),
        issued: Utc::now().to_rfc3339(),
        value_quantity: FhirQuantity {
            value,
            unit: unit_display.to_string(),
            system: systems::UCUM.to_string(),
            code: unit_code.to_string(),
        },
        interpretation: None,
        reference_range: get_reference_range(obs_type),
        device: Some(FhirReference {
            reference: format!("Device/esms-sensor-{:?}", obs_type).to_lowercase(),
            display: Some(get_device_display(obs_type)),
        }),
        note: Some(vec![FhirAnnotation {
            text: "This observation is from the Environmental Stress Monitoring System. \
                   It identifies environmental and physiological conditions that correlate \
                   with stress and discomfort. It does NOT perform diagnosis and is intended \
                   as an early-warning monitoring tool.".to_string(),
            time: Utc::now().to_rfc3339(),
        }]),
    })
}

/// Convert a sensor reading to multiple FHIR Observations (one per metric)
pub fn to_fhir_bundle(
    reading: &SensorReading,
    patient_reference: &str,
) -> AppResult<FhirBundle> {
    let observations = vec![
        to_fhir_observation(reading, ObservationType::Temperature, patient_reference)?,
        to_fhir_observation(reading, ObservationType::Humidity, patient_reference)?,
        to_fhir_observation(reading, ObservationType::SoundLevel, patient_reference)?,
        to_fhir_observation(reading, ObservationType::HeartRate, patient_reference)?,
    ];

    Ok(FhirBundle {
        resource_type: "Bundle".to_string(),
        id: Uuid::new_v4().to_string(),
        bundle_type: "collection".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        total: observations.len() as u32,
        entry: observations
            .into_iter()
            .map(|obs| FhirBundleEntry {
                full_url: format!("urn:uuid:{}", obs.id),
                resource: obs,
            })
            .collect(),
    })
}

/// FHIR Bundle resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirBundle {
    pub resource_type: String,
    pub id: String,
    #[serde(rename = "type")]
    pub bundle_type: String,
    pub timestamp: String,
    pub total: u32,
    pub entry: Vec<FhirBundleEntry>,
}

/// FHIR Bundle entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirBundleEntry {
    pub full_url: String,
    pub resource: FhirObservation,
}

fn get_reference_range(obs_type: ObservationType) -> Option<Vec<FhirReferenceRange>> {
    match obs_type {
        ObservationType::Temperature => Some(vec![FhirReferenceRange {
            low: Some(FhirQuantity {
                value: 18.0,
                unit: "°C".to_string(),
                system: systems::UCUM.to_string(),
                code: "Cel".to_string(),
            }),
            high: Some(FhirQuantity {
                value: 26.0,
                unit: "°C".to_string(),
                system: systems::UCUM.to_string(),
                code: "Cel".to_string(),
            }),
            text: Some("Comfortable room temperature range".to_string()),
        }]),
        ObservationType::Humidity => Some(vec![FhirReferenceRange {
            low: Some(FhirQuantity {
                value: 30.0,
                unit: "%".to_string(),
                system: systems::UCUM.to_string(),
                code: "%".to_string(),
            }),
            high: Some(FhirQuantity {
                value: 60.0,
                unit: "%".to_string(),
                system: systems::UCUM.to_string(),
                code: "%".to_string(),
            }),
            text: Some("Comfortable humidity range".to_string()),
        }]),
        ObservationType::HeartRate => Some(vec![FhirReferenceRange {
            low: Some(FhirQuantity {
                value: 60.0,
                unit: "beats/minute".to_string(),
                system: systems::UCUM.to_string(),
                code: "/min".to_string(),
            }),
            high: Some(FhirQuantity {
                value: 100.0,
                unit: "beats/minute".to_string(),
                system: systems::UCUM.to_string(),
                code: "/min".to_string(),
            }),
            text: Some("Normal resting heart rate range".to_string()),
        }]),
        ObservationType::SoundLevel => Some(vec![FhirReferenceRange {
            low: None,
            high: Some(FhirQuantity {
                value: 400.0,
                unit: "units".to_string(),
                system: systems::UCUM.to_string(),
                code: "1".to_string(),
            }),
            text: Some("Comfortable ambient noise level".to_string()),
        }]),
    }
}

fn get_device_display(obs_type: ObservationType) -> String {
    match obs_type {
        ObservationType::Temperature | ObservationType::Humidity => {
            "DHT11 Temperature and Humidity Sensor".to_string()
        }
        ObservationType::SoundLevel => "Sound Level Sensor Module".to_string(),
        ObservationType::HeartRate => "MAX30100 Pulse Oximeter and Heart Rate Sensor".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_observation() {
        let reading = SensorReading::new(25.5, 55.0, 300.0, 72.0);
        let obs = to_fhir_observation(&reading, ObservationType::Temperature, "Patient/test")
            .unwrap();

        assert_eq!(obs.resource_type, "Observation");
        assert_eq!(obs.status, "final");
        assert_eq!(obs.code.coding[0].code, loinc::TEMPERATURE);
        assert_eq!(obs.value_quantity.value, 25.5);
        assert_eq!(obs.value_quantity.unit, "°C");
    }

    #[test]
    fn test_heart_rate_observation() {
        let reading = SensorReading::new(25.0, 55.0, 300.0, 85.0);
        let obs = to_fhir_observation(&reading, ObservationType::HeartRate, "Patient/test")
            .unwrap();

        assert_eq!(obs.code.coding[0].code, loinc::HEART_RATE);
        assert_eq!(obs.code.coding[0].system, systems::LOINC);
        assert_eq!(obs.value_quantity.value, 85.0);
    }

    #[test]
    fn test_fhir_bundle() {
        let reading = SensorReading::new(25.0, 55.0, 300.0, 72.0);
        let bundle = to_fhir_bundle(&reading, "Patient/test").unwrap();

        assert_eq!(bundle.resource_type, "Bundle");
        assert_eq!(bundle.bundle_type, "collection");
        assert_eq!(bundle.total, 4);
        assert_eq!(bundle.entry.len(), 4);
    }

    #[test]
    fn test_observation_disclaimer() {
        let reading = SensorReading::new(25.0, 55.0, 300.0, 72.0);
        let obs = to_fhir_observation(&reading, ObservationType::Temperature, "Patient/test")
            .unwrap();

        let note = obs.note.unwrap();
        assert!(!note.is_empty());
        assert!(note[0].text.contains("NOT perform diagnosis"));
    }
}
