//! WebSocket module for real-time sensor data streaming
//!
//! Provides a WebSocket endpoint for clients to receive real-time sensor updates.

use actix::{
    Actor, ActorContext, ActorFutureExt, AsyncContext, Message, StreamHandler,
};
use actix_web_actors::ws;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::models::{SensorReading, WsMessage};
use crate::state::AppState;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// WebSocket session actor
pub struct WsSession {
    client_id: String,
    last_heartbeat: Instant,
    state: Arc<RwLock<AppState>>,
    last_reading_id: Option<String>,
}

impl WsSession {
    pub fn new(client_id: String, state: Arc<RwLock<AppState>>) -> Self {
        Self {
            client_id,
            last_heartbeat: Instant::now(),
            state,
            last_reading_id: None,
        }
    }

    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                warn!(
                    client_id = %act.client_id,
                    "WebSocket heartbeat timeout"
                );
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    fn start_sensor_polling(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(1), |act, ctx| {
            let state = act.state.clone();

            let fut = async move {
                let state = state.read().await;
                state.get_latest().cloned()
            };

            let fut = actix::fut::wrap_future::<_, Self>(fut);

            ctx.spawn(fut.map(|reading, act, ctx| {
                if let Some(reading) = reading {
                    let reading_id = reading.id.to_string();

                    if act.last_reading_id.as_ref() != Some(&reading_id) {
                        act.last_reading_id = Some(reading_id);

                        let msg = WsMessage::SensorUpdate(reading);
                        if let Ok(json) = serde_json::to_string(&msg) {
                            ctx.text(json);
                        }
                    }
                }
            }));
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(client_id = %self.client_id, "WebSocket connected");

        self.start_heartbeat(ctx);
        self.start_sensor_polling(ctx);

        let msg = WsMessage::Connected {
            client_id: self.client_id.clone(),
        };

        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!(client_id = %self.client_id, "WebSocket disconnected");

        let state = self.state.clone();
        let client_id = self.client_id.clone();

        // IMPORTANT: Actix runtime spawn (not Tokio)
        actix_rt::spawn(async move {
            let mut state = state.write().await;
            state.remove_client(&client_id);
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                debug!(client_id = %self.client_id, message = %text);

                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(WsMessage::Ping) => {
                        self.last_heartbeat = Instant::now();
                        if let Ok(json) = serde_json::to_string(&WsMessage::Pong) {
                            ctx.text(json);
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!(client_id = %self.client_id, error = %e);
                        let err = WsMessage::Error {
                            message: "Invalid message format".into(),
                        };
                        if let Ok(json) = serde_json::to_string(&err) {
                            ctx.text(json);
                        }
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                info!(client_id = %self.client_id, reason = ?reason);
                ctx.stop();
            }
            Err(e) => {
                warn!(client_id = %self.client_id, error = %e);
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// Message to broadcast sensor update
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastReading(pub SensorReading);
