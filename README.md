# Environmental Stress Monitoring System (ESMS)

> Real-time monitoring of environmental and physiological factors correlated with stress and discomfort.

**⚠️ DISCLAIMER:** This system identifies environmental and physiological conditions that correlate with increased stress and discomfort. It does NOT perform diagnosis and is intended as an early-warning monitoring tool.

## 🏗️ Architecture

```
┌─────────────────────────┐     ┌─────────────────────────────────────────┐
│  Fake Sensor Generator  │     │           Rust Backend (Actix)          │
│  (src/fake_sensor.rs)   │────▶│  ├─ REST API (/api/sensor/*)            │
│  DHT11, Sound, MAX30100 │     │  ├─ WebSocket (/ws)                     │
└─────────────────────────┘     │  ├─ FHIR R4 (/api/fhir/Observation/*)   │
                                │  └─ Health Check (/api/health)          │
                                └──────────────────┬──────────────────────┘
                                                   │ WebSocket
                                ┌──────────────────▼──────────────────────┐
                                │        D3.js Frontend Dashboard         │
                                │  Real-time charts for all 4 sensors     │
                                └─────────────────────────────────────────┘
```

## 🔬 Sensors

| Sensor | Measurement | LOINC Code |
|--------|-------------|------------|
| DHT11 | Temperature (°C) | 8310-5 |
| DHT11 | Humidity (%) | ESMS-ENV-001 |
| Sound Level | Ambient Noise | ESMS-ENV-002 |
| MAX30100 | Heart Rate (BPM) | 8867-4 |

## 🚀 Quick Start

```bash
# Backend
cd esms-backend
cp .env.example .env
cargo run

# Frontend (separate terminal)
cd esms-frontend
python -m http.server 3000
# Open http://localhost:3000
```

## 🐳 Docker

```bash
cd esms-backend
docker-compose up --build
```

## 📡 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/sensor/ingest` | Ingest sensor data |
| GET | `/api/sensor/latest` | Latest reading |
| GET | `/api/sensor/history` | Historical data |
| GET | `/api/fhir/Observation/latest` | FHIR Bundle |
| WS | `/ws` | Real-time stream |

## 🏥 FHIR Compliance

All sensor readings are convertible to FHIR R4 Observation resources with proper LOINC coding.

## 📁 Project Structure

```
esms-backend/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── config.rs         # Configuration
│   ├── error.rs          # Error handling
│   ├── models.rs         # Data models
│   ├── fake_sensor.rs    # Fake data generator
│   ├── state.rs          # Application state
│   ├── validation.rs     # Input validation
│   ├── fhir.rs           # FHIR conversion
│   ├── handlers.rs       # HTTP handlers
│   └── websocket.rs      # WebSocket
├── Dockerfile
└── docker-compose.yml

esms-frontend/
├── index.html
├── styles.css
└── app.js
```

## 📜 License

MIT