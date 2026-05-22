//! Lyria RealTime session manager.
//!
//! Manages a WebSocket connection to the Lyria RealTime API for
//! interactive, streaming music generation.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

const MODEL: &str = "models/lyria-realtime-exp";
const WS_BASE: &str = "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1alpha.GenerativeService.BidiGenerateMusic";

/// Session state.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Connected,
    Playing,
    Paused,
    Stopped,
}

/// A managed Lyria RealTime session.
pub struct RealtimeSession {
    /// Session ID
    pub id: String,
    /// Current state
    pub state: Arc<RwLock<SessionState>>,
    /// Accumulated PCM audio data (48kHz, stereo, 16-bit)
    audio_buffer: Arc<Mutex<Vec<u8>>>,
    /// WebSocket sender
    ws_tx: Arc<Mutex<Option<futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >>>>,
    /// Background task handle
    _recv_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl RealtimeSession {
    /// Connect and create a new session.
    pub async fn connect(api_key: &str, prompts: Vec<WeightedPrompt>, config: Option<MusicGenConfig>) -> Result<Self, String> {
        let url = format!("{}?key={}", WS_BASE, api_key);
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| format!("WebSocket connect failed: {}", e))?;

        let (mut ws_tx, mut ws_rx) = ws_stream.split();
        let session_id = uuid::Uuid::new_v4().to_string();

        // Send setup message
        let setup = serde_json::json!({
            "setup": {
                "model": MODEL
            }
        });
        ws_tx.send(Message::Text(setup.to_string().into())).await
            .map_err(|e| format!("Setup send failed: {}", e))?;

        // Wait for setup complete with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(10), ws_rx.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                debug!(msg = %text, "Setup response received");
                // Check if it's an error
                if text.contains("error") {
                    return Err(format!("Setup rejected: {}", text));
                }
            }
            Ok(Some(Ok(Message::Close(frame)))) => {
                let reason = frame.map(|f| f.reason.to_string()).unwrap_or_default();
                return Err(format!("Server closed connection during setup: {}", reason));
            }
            Ok(Some(Err(e))) => return Err(format!("Setup response error: {}", e)),
            Ok(None) => return Err("Connection closed before setup response".to_string()),
            Err(_) => return Err("Timeout waiting for setup response".to_string()),
            _ => { debug!("Unexpected setup response type"); }
        }

        let audio_buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let state: Arc<RwLock<SessionState>> = Arc::new(RwLock::new(SessionState::Connected));

        let ws_tx = Arc::new(Mutex::new(Some(ws_tx)));

        // Spawn receiver task
        let buf_clone = audio_buffer.clone();
        let state_clone = state.clone();
        let recv_task = tokio::spawn(async move {
            while let Some(msg_result) = ws_rx.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        // Parse for audio chunks
                        let s = text.to_string();
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                            if let Some(data) = val.pointer("/serverContent/audioChunks/0/data")
                                .and_then(|d| d.as_str())
                            {
                                if let Ok(bytes) = BASE64.decode(data) {
                                    buf_clone.lock().await.extend_from_slice(&bytes);
                                }
                            }
                        } else {
                            // If JSON parse fails, store raw for debugging
                            error!("Failed to parse audio message");
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // Server may send JSON as binary frames
                        if let Ok(s) = std::str::from_utf8(&data) {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
                                if let Some(audio_data) = val.pointer("/serverContent/audioChunks/0/data")
                                    .and_then(|d| d.as_str())
                                {
                                    if let Ok(bytes) = BASE64.decode(audio_data) {
                                        buf_clone.lock().await.extend_from_slice(&bytes);
                                    }
                                }
                            } else {
                                buf_clone.lock().await.extend_from_slice(&data);
                            }
                        } else {
                            buf_clone.lock().await.extend_from_slice(&data);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket closed by server");
                        *state_clone.write().await = SessionState::Stopped;
                        break;
                    }
                    Err(e) => {
                        error!(error = %e, "WebSocket receive error");
                        *state_clone.write().await = SessionState::Stopped;
                        break;
                    }
                    _ => {}
                }
            }
        });

        let session = Self {
            id: session_id,
            state,
            audio_buffer,
            ws_tx,
            _recv_task: Arc::new(Mutex::new(Some(recv_task))),
        };

        // Small delay to let the connection stabilize
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Send config first if provided
        if let Some(cfg) = config {
            session.send_config(&cfg).await?;
        }

        // Send initial prompts
        session.send_prompts(&prompts).await?;

        // Start playing
        session.send_play().await?;

        Ok(session)
    }

    /// Send weighted prompts.
    pub async fn send_prompts(&self, prompts: &[WeightedPrompt]) -> Result<(), String> {
        let msg = serde_json::json!({
            "client_content": {
                "weightedPrompts": prompts.iter().map(|p| {
                    serde_json::json!({"text": p.text, "weight": p.weight})
                }).collect::<Vec<_>>()
            }
        });
        self.send_json(&msg).await
    }

    /// Send music generation config.
    pub async fn send_config(&self, config: &MusicGenConfig) -> Result<(), String> {
        let msg = serde_json::json!({
            "music_generation_config": config
        });
        self.send_json(&msg).await
    }

    /// Send play command.
    pub async fn send_play(&self) -> Result<(), String> {
        let msg = serde_json::json!({
            "playback_control": "PLAY"
        });
        *self.state.write().await = SessionState::Playing;
        self.send_json(&msg).await
    }

    /// Send pause command.
    pub async fn send_pause(&self) -> Result<(), String> {
        let msg = serde_json::json!({
            "playback_control": "PAUSE"
        });
        *self.state.write().await = SessionState::Paused;
        self.send_json(&msg).await
    }

    /// Send stop command and close the connection.
    pub async fn stop(&self) -> Result<Vec<u8>, String> {
        let msg = serde_json::json!({
            "playback_control": "STOP"
        });
        let _ = self.send_json(&msg).await;
        *self.state.write().await = SessionState::Stopped;

        // Close WebSocket
        if let Some(mut tx) = self.ws_tx.lock().await.take() {
            let _ = tx.close().await;
        }

        // Return accumulated audio
        let audio = self.audio_buffer.lock().await.clone();
        Ok(audio)
    }

    /// Get current buffer size in bytes.
    pub async fn buffer_size(&self) -> usize {
        self.audio_buffer.lock().await.len()
    }

    /// Send a JSON message over WebSocket.
    pub async fn send_json_public(&self, msg: &serde_json::Value) -> Result<(), String> {
        self.send_json(msg).await
    }

    /// Send a JSON message over WebSocket.
    async fn send_json(&self, msg: &serde_json::Value) -> Result<(), String> {
        let mut guard = self.ws_tx.lock().await;
        if let Some(ref mut tx) = *guard {
            tx.send(Message::Text(msg.to_string().into())).await
                .map_err(|e| format!("WebSocket send failed: {}", e))
        } else {
            Err("WebSocket closed".to_string())
        }
    }
}

/// Weighted prompt for steering.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WeightedPrompt {
    /// Text prompt (genre, instrument, mood, etc.)
    pub text: String,
    /// Weight (non-zero, 1.0 is default)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 { 1.0 }

/// Music generation config for real-time steering.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MusicGenConfig {
    /// BPM (60-200)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bpm: Option<u16>,
    /// Temperature (0.0-3.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Guidance (0.0-6.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance: Option<f64>,
    /// Density (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<f64>,
    /// Brightness (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness: Option<f64>,
    /// Musical scale
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
    /// Mute bass
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_bass: Option<bool>,
    /// Mute drums
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_drums: Option<bool>,
    /// Only bass and drums
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_bass_and_drums: Option<bool>,
    /// Generation mode: QUALITY, DIVERSITY, or VOCALIZATION
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_generation_mode: Option<String>,
}

/// Session manager holding active sessions.
pub struct SessionManager {
    sessions: Arc<RwLock<std::collections::HashMap<String, Arc<RealtimeSession>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn create_session(
        &self,
        api_key: &str,
        prompts: Vec<WeightedPrompt>,
        config: Option<MusicGenConfig>,
    ) -> Result<String, String> {
        let session = RealtimeSession::connect(api_key, prompts, config).await?;
        let id = session.id.clone();
        self.sessions.write().await.insert(id.clone(), Arc::new(session));
        Ok(id)
    }

    pub async fn get_session(&self, id: &str) -> Option<Arc<RealtimeSession>> {
        self.sessions.read().await.get(id).cloned()
    }

    pub async fn remove_session(&self, id: &str) -> Option<Arc<RealtimeSession>> {
        self.sessions.write().await.remove(id)
    }
}
