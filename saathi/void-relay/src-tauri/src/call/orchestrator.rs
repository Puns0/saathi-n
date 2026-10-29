use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::ai::engine_router::EngineRouter;
use crate::ai::language_detector::LanguageCode;
use crate::ai::llm::llm_router::Message;
use crate::bridge::ws_server::{WsServer, WsCommand};
use crate::config::settings::AppSettings;
use crate::audio::pipeline::AudioPipeline;
use crate::audio::vad::Vad;
use crate::bluetooth::audio_bridge::AudioBridge;

use super::history::{add_record, CallRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CallState {
    Idle,
    Initiating,
    Ringing,
    Connected,
    Listening,
    Processing,
    Speaking,
    Ended,
}

pub struct ActiveCall {
    pub id: String,
    pub number: String,
    pub platform: String,
    pub state: CallState,
    pub start_time: std::time::Instant,
    pub history: Vec<Message>,
    pub language: LanguageCode,
}

#[derive(Clone)]
pub struct CallOrchestrator {
    app_handle: tauri::AppHandle,
    ws_server: Arc<WsServer>,
    engine_router: Arc<RwLock<EngineRouter>>,
    db: Arc<parking_lot::Mutex<rusqlite::Connection>>,
    settings: Arc<RwLock<AppSettings>>,
    active_call: Arc<RwLock<Option<ActiveCall>>>,
    audio_pipeline: Arc<RwLock<Option<AudioPipeline>>>,
    audio_bridge: Arc<AudioBridge>,
}

impl CallOrchestrator {
    pub fn new(
        app_handle: tauri::AppHandle,
        ws_server: Arc<WsServer>, 
        engine_router: Arc<RwLock<EngineRouter>>,
        db: Arc<parking_lot::Mutex<rusqlite::Connection>>,
        settings: Arc<RwLock<AppSettings>>
    ) -> Self {
        Self {
            app_handle,
            ws_server,
            engine_router,
            db,
            settings,
            active_call: Arc::new(RwLock::new(None)),
            audio_pipeline: Arc::new(RwLock::new(None)),
            audio_bridge: Arc::new(AudioBridge::new()),
        }
    }

    pub fn get_state(&self) -> CallState {
        self.active_call.read().as_ref().map(|c| c.state).unwrap_or(CallState::Idle)
    }

    pub async fn initiate_call(&self, number: String, platform_hint: Option<String>) -> anyhow::Result<String> {
        if self.get_state() != CallState::Idle {
            anyhow::bail!("A call is already active");
        }

        let platform = match platform_hint {
            Some(p) => p,
            None => {
                let s = self.settings.read();
                let pref = s.preferred_phone.clone();
                let connected = self.ws_server.get_connected_platforms();
                
                if pref == "auto" {
                    if connected.contains(&"android".to_string()) {
                        "android".to_string()
                    } else if connected.contains(&"ios".to_string()) {
                        "ios".to_string()
                    } else {
                        // Allow call even without phone connected (for audio testing)
                        "none".to_string()
                    }
                } else {
                    pref
                }
            }
        };        let call_id = uuid::Uuid::new_v4().to_string();
        
        {
            let mut active = self.active_call.write();
            *active = Some(ActiveCall {
                id: call_id.clone(),
                number: number.clone(),
                platform: platform.clone(),
                state: CallState::Connected,
                start_time: std::time::Instant::now(),
                history: Vec::new(),
                language: LanguageCode::English,
            });
        }

        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": "CONNECTED" }));
        let _ = self.app_handle.emit("call_started", serde_json::json!({
            "number": number.clone(),
            "platform": platform.clone()
        }));

        // Start audio pipeline immediately — don't wait for phone confirmation
        self.start_audio_pipeline().await;

        // Send command to phone if connected
        if platform != "none" {
            let _ = self.ws_server.send_command(&platform, WsCommand::Call {
                number: number.clone(),
                call_id: call_id.clone(),
            }).await;
        }

        Ok(call_id)
    }

    /// Starts the audio pipeline using configured or default devices.
    /// Safe to call multiple times — stops existing pipeline first.
    pub async fn start_audio_pipeline(&self) {
        // Stop any existing pipeline
        if let Some(mut p) = self.audio_pipeline.write().take() {
            p.stop();
        }

        let (tx, mut rx) = mpsc::channel::<Vec<i16>>(32);

        let (vad_sensitivity, silence_timeout_ms, input_device, output_device) = {
            let s = self.settings.read();
            (s.vad_sensitivity, s.silence_timeout_ms,
             s.audio_input_device.clone(), s.audio_output_device.clone())
        };

        let mut pipeline = AudioPipeline::new();
        let vad = Vad::new(vad_sensitivity, 300, silence_timeout_ms, 8000, pipeline.suppress_capture.clone());

        match pipeline.start_with_names(&input_device, &output_device, tx, vad) {
            Ok(_) => {
                log::info!("Audio pipeline started — IN:'{}' OUT:'{}'",
                    if input_device.is_empty() { "default" } else { &input_device },
                    if output_device.is_empty() { "default" } else { &output_device }
                );
                *self.audio_pipeline.write() = Some(pipeline);
                self.set_call_state(CallState::Listening);

                // Spawn STT processing loop
                let orch = self.clone();
                tauri::async_runtime::spawn(async move {
                    while let Some(audio) = rx.recv().await {
                        orch.process_audio(audio).await;
                    }
                });

                // Play a TTS greeting to confirm speakers work
                let orch = self.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    orch.say_text("Hello, AI agent is ready. I am listening.".to_string()).await.ok();
                });
            }
            Err(e) => {
                log::error!("Failed to start audio pipeline: {}", e);
            }
        }
    }
    pub async fn hangup(&self) -> anyhow::Result<()> {
        let call_info = {
            let mut active = self.active_call.write();
            let info = active.take();
            if let Some(mut pipeline) = self.audio_pipeline.write().take() {
                pipeline.stop();
            }
            info
        };

        if let Some(call) = call_info {
            if call.platform != "none" {
                let _ = self.ws_server.send_command(&call.platform, WsCommand::Hangup { call_id: call.id }).await;
            }
        }
        
        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": "ENDED" }));
        
        Ok(())
    }
    pub async fn handle_call_started(&self, number: String, call_id: String, platform: String) {
        {
            let mut active = self.active_call.write();
            if let Some(call) = active.as_mut() {
                if call.id == call_id {
                    call.state = CallState::Connected;
                }
            } else {
                // Incoming call answered without prior state? Or Initiating state missed.
                *active = Some(ActiveCall {
                    id: call_id.clone(),
                    number: number.clone(),
                    platform: platform.clone(),
                    state: CallState::Connected,
                    start_time: std::time::Instant::now(),
                    history: Vec::new(),
                    language: LanguageCode::English,
                });
            }
        }

        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": "CONNECTED" }));
        let _ = self.app_handle.emit("call_started", serde_json::json!({
            "number": number,
            "platform": platform
        }));

        // Start (or restart) audio pipeline using system/configured devices
        self.start_audio_pipeline().await;
    }

    pub async fn handle_incoming_call(&self, number: String, platform: String) {
        let mut active = self.active_call.write();
        if active.is_none() {
            *active = Some(ActiveCall {
                id: uuid::Uuid::new_v4().to_string(),
                number: number.clone(),
                platform: platform.clone(),
                state: CallState::Ringing,
                start_time: std::time::Instant::now(),
                history: Vec::new(),
                language: LanguageCode::English,
            });
        }
        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": "RINGING" }));
        let _ = self.app_handle.emit("call_started", serde_json::json!({
            "number": number,
            "platform": platform
        }));
    }

    pub async fn handle_call_ended(&self, call_id: String) {
        let call_info = {
            let mut active = self.active_call.write();
            let info = active.take();
            if let Some(mut pipeline) = self.audio_pipeline.write().take() {
                pipeline.stop();
            }
            info
        };

        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": "ENDED" }));

        if let Some(call) = call_info {
            let duration = call.start_time.elapsed().as_secs() as u32;
            
            // Format transcript
            let mut full_transcript = String::new();
            for msg in &call.history {
                full_transcript.push_str(&format!("{}: {}\n", msg.role, msg.content));
            }

            // Summarize using the Groq LLM client
            let summary = if full_transcript.is_empty() {
                "No conversation recorded.".to_string()
            } else {
                let llm_router = {
                    let engine = self.engine_router.read();
                    engine.llm_router.clone()
                };
                match llm_router.summarize(&full_transcript).await {
                    Ok(sum) => sum,
                    Err(e) => {
                        log::error!("Failed to generate Groq call summary: {}", e);
                        "Failed to generate summary.".to_string()
                    }
                }
            };

            let record = CallRecord {
                id: call.id,
                timestamp: chrono::Utc::now().to_rfc3339(),
                number: call.number,
                duration_sec: duration,
                platform: call.platform,
                language: call.language.as_str().to_string(),
                summary,
                full_transcript,
            };

            if let Err(e) = add_record(&self.db, &record) {
                log::error!("Failed to save call history: {}", e);
            }
        }
    }

    fn set_call_state(&self, state: CallState) {
        if let Some(call) = self.active_call.write().as_mut() {
            call.state = state;
        }
        let state_str = match state {
            CallState::Idle => "IDLE",
            CallState::Initiating => "INITIATING",
            CallState::Ringing => "RINGING",
            CallState::Connected => "CONNECTED",
            CallState::Listening => "LISTENING",
            CallState::Processing => "PROCESSING",
            CallState::Speaking => "SPEAKING",
            CallState::Ended => "ENDED",
        };
        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": state_str }));
    }

    // Process a chunk of speech returned from VAD
    pub async fn process_audio(&self, audio: Vec<i16>) {
        self.set_call_state(CallState::Processing);
        
        let (mode, online, stt_router, llm_router, tts_router, language_detector) = {
            let engine = self.engine_router.read();
            let (m, o) = engine.get_mode();
            (m, o, engine.stt_router.clone(), engine.llm_router.clone(), engine.tts_router.clone(), engine.language_detector.clone())
        };

        let lang = language_detector.detect(&audio).await;
        
        {
            if let Some(call) = self.active_call.write().as_mut() {
                call.language = lang;
            }
        }

        let stt_result = stt_router.transcribe(&audio, lang, online, mode).await;
        
        match stt_result {
            Ok(transcript) => {
                log::info!("Caller: {}", transcript);
                // 1. Add to history
                let history = {
                    let mut active = self.active_call.write();
                    if let Some(call) = active.as_mut() {
                        call.history.push(Message {
                            role: "user".to_string(),
                            content: transcript.clone(),
                        });
                        call.history.clone()
                    } else {
                        return;
                    }
                };

                let _ = self.app_handle.emit("stt_transcript", serde_json::json!({ "text": transcript }));

                // 2. Stream LLM
                match llm_router.chat(transcript, &history, online, mode).await {
                    Ok(mut stream) => {
                        self.set_call_state(CallState::Speaking);
                        let mut full_response = String::new();
                        let mut sentence_buffer = String::new();
                        
                        while let Some(token) = stream.next().await {
                            full_response.push_str(&token);
                            sentence_buffer.push_str(&token);
                            
                            let _ = self.app_handle.emit("llm_token", serde_json::json!({ "token": token }));

                            // Basic sentence boundary detection
                            if token.contains('.') || token.contains('?') || token.contains('!') {
                                // 3. TTS for sentence
                                let tts_text = sentence_buffer.trim().to_string();
                                sentence_buffer.clear();
                                
                                if !tts_text.is_empty() {
                                    match tts_router.synthesize(tts_text, lang, online, mode).await {
                                        Ok(audio) => {
                                            if let Some(pipeline) = self.audio_pipeline.write().as_mut() {
                                                pipeline.play(&audio);
                                            }
                                        }
                                        Err(e) => log::error!("TTS error: {}", e),
                                    }
                                }
                            }
                        }
                        
                        // Remaining buffer
                        let tts_text = sentence_buffer.trim().to_string();
                        if !tts_text.is_empty() {
                            if let Ok(audio) = tts_router.synthesize(tts_text, lang, online, mode).await {
                                if let Some(pipeline) = self.audio_pipeline.write().as_mut() {
                                    pipeline.play(&audio);
                                }
                            }
                        }
                        
                        let _ = self.app_handle.emit("llm_finished", serde_json::json!({ "text": full_response }));

                        // Add AI response to history
                        if let Some(call) = self.active_call.write().as_mut() {
                            call.history.push(Message {
                                role: "assistant".to_string(),
                                content: full_response,
                            });
                        }
                    }
                    Err(e) => log::error!("LLM error: {}", e),
                }
            }
            Err(e) => log::error!("STT error: {}", e),
        }
        
        self.set_call_state(CallState::Listening);
    }
    
    pub async fn say_text(&self, text: String) -> anyhow::Result<()> {
        let (mode, online, tts_router) = {
            let engine = self.engine_router.read();
            let (m, o) = engine.get_mode();
            (m, o, engine.tts_router.clone())
        };
        
        let lang = {
            self.active_call.read().as_ref().map(|c| c.language).unwrap_or(LanguageCode::English)
        };
        
        let audio = tts_router.synthesize(text.clone(), lang, online, mode).await?;
        if let Some(pipeline) = self.audio_pipeline.write().as_mut() {
            pipeline.play(&audio);
        }
        
        let _ = self.app_handle.emit("llm_finished", serde_json::json!({ "text": text }));

        if let Some(call) = self.active_call.write().as_mut() {
            call.history.push(Message {
                role: "assistant".to_string(),
                content: text,
            });
        }
        
        Ok(())
    }
}
