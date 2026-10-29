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
    Screening,
    Handoff,
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
    pub is_screening: bool,
    pub caller_spoke: Arc<std::sync::atomic::AtomicBool>,
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
                is_screening: false,
                caller_spoke: Arc::new(std::sync::atomic::AtomicBool::new(false)),
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

        let (vad_sensitivity, silence_timeout_ms, half_duplex_tail_ms, input_device, output_device) = {
            let s = self.settings.read();
            (s.vad_sensitivity, s.silence_timeout_ms, s.half_duplex_tail_ms,
             s.audio_input_device.clone(), s.audio_output_device.clone())
        };

        let mut pipeline = AudioPipeline::with_app_handle(self.app_handle.clone());
        pipeline.half_duplex_tail_ms = if half_duplex_tail_ms == 0 { 400 } else { half_duplex_tail_ms };
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

                let is_screening = self.active_call.read().as_ref().map(|c| c.is_screening).unwrap_or(false);
                if !is_screening {
                    // Outbound / manual call: play a brief TTS greeting to confirm speakers work
                    let orch = self.clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        orch.say_text("Hello, AI agent is ready. I am listening.".to_string()).await.ok();
                    });
                }
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

    pub async fn take_over_call(&self) -> anyhow::Result<()> {
        log::info!("Human requested call takeover (Handoff)");
        if let Some(mut pipeline) = self.audio_pipeline.write().take() {
            pipeline.stop();
        }
        self.set_call_state(CallState::Handoff);
        Ok(())
    }

    pub async fn handle_call_started(&self, number: String, call_id: String, platform: String) {
        let (is_screening, caller_spoke) = {
            let mut active = self.active_call.write();
            if let Some(call) = active.as_mut() {
                call.id = call_id.clone();
                call.state = if call.is_screening { CallState::Screening } else { CallState::Connected };
                (call.is_screening, call.caller_spoke.clone())
            } else {
                let caller_spoke = Arc::new(std::sync::atomic::AtomicBool::new(false));
                *active = Some(ActiveCall {
                    id: call_id.clone(),
                    number: number.clone(),
                    platform: platform.clone(),
                    state: CallState::Connected,
                    start_time: std::time::Instant::now(),
                    history: Vec::new(),
                    language: LanguageCode::English,
                    is_screening: false,
                    caller_spoke: caller_spoke.clone(),
                });
                (false, caller_spoke)
            }
        };

        let state_str = if is_screening { "SCREENING" } else { "CONNECTED" };
        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": state_str }));
        let _ = self.app_handle.emit("call_started", serde_json::json!({
            "number": number,
            "platform": platform,
            "screening": is_screening
        }));

        // Start (or restart) audio pipeline using system/configured devices
        self.start_audio_pipeline().await;

        if is_screening {
            let orch = self.clone();
            let user_name = self.settings.read().user_name.clone();
            tauri::async_runtime::spawn(async move {
                // Wait 1.5s for caller to speak first
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                if !caller_spoke.load(std::sync::atomic::Ordering::SeqCst) {
                    let name_str = if user_name.trim().is_empty() { "the user".to_string() } else { user_name };
                    let greeting = format!("Hello, this is an AI assistant answering for {}. Who is calling, and what is it about?", name_str);
                    log::info!("Caller silent after 1.5s, playing screening greeting: {}", greeting);
                    let _ = orch.say_text(greeting).await;
                }
            });
        }
    }

    pub async fn handle_incoming_call(&self, number: String, platform: String, known: Option<bool>, contact_name: Option<String>) {
        let is_known = known.unwrap_or(false);
        let is_screening = !is_known;
        let caller_spoke = Arc::new(std::sync::atomic::AtomicBool::new(false));

        {
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
                    is_screening,
                    caller_spoke,
                });
            } else if let Some(call) = active.as_mut() {
                call.is_screening = is_screening;
            }
        }
        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": "RINGING" }));
        let _ = self.app_handle.emit("incoming_call", serde_json::json!({
            "number": number,
            "platform": platform,
            "known": is_known,
            "contact_name": contact_name
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
            CallState::Screening => "SCREENING",
            CallState::Handoff => "HANDOFF",
            CallState::Listening => "LISTENING",
            CallState::Processing => "PROCESSING",
            CallState::Speaking => "SPEAKING",
            CallState::Ended => "ENDED",
        };
        let _ = self.app_handle.emit("call_state_changed", serde_json::json!({ "state": state_str }));
    }

    // Process a chunk of speech returned from VAD
    pub async fn process_audio(&self, audio: Vec<i16>) {
        let turn_start = std::time::Instant::now();
        self.set_call_state(CallState::Processing);

        // Mark caller as having spoken (used to cancel 1.5s initial greeting timer)
        if let Some(call) = self.active_call.read().as_ref() {
            call.caller_spoke.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        
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

        let stt_start = std::time::Instant::now();
        let stt_result = stt_router.transcribe(&audio, lang, online, mode).await;
        let stt_done_ms = stt_start.elapsed().as_millis() as u64;
        
        match stt_result {
            Ok(transcript) => {
                log::info!("Caller: {} (STT: {}ms)", transcript, stt_done_ms);
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

                let llm_start = std::time::Instant::now();
                let mut first_token_ms: Option<u64> = None;
                let mut first_audio_ms: Option<u64> = None;

                // 2. Stream LLM with 1.0s filler threshold
                match llm_router.chat(transcript.clone(), &history, online, mode).await {
                    Ok(mut stream) => {
                        self.set_call_state(CallState::Speaking);
                        let mut full_response = String::new();
                        let mut sentence_buffer = String::new();

                        // If first token takes > 1000ms, play filler
                        let first_token_fut = stream.next();
                        let first_token = match tokio::time::timeout(tokio::time::Duration::from_millis(1000), first_token_fut).await {
                            Ok(token_opt) => token_opt,
                            Err(_) => {
                                let filler = match lang {
                                    LanguageCode::Hindi | LanguageCode::Mixed => "Ek minute...",
                                    LanguageCode::Kannada => "Ondu nimisha...",
                                    _ => "One moment...",
                                };
                                log::info!("LLM response delayed > 1s, sending filler: {}", filler);
                                if let Ok(filler_audio) = tts_router.synthesize(filler.to_string(), lang, online, mode).await {
                                    if let Some(pipeline) = self.audio_pipeline.write().as_mut() {
                                        pipeline.play(&filler_audio);
                                    }
                                }
                                stream.next().await
                            }
                        };

                        if let Some(token) = first_token {
                            first_token_ms = Some(llm_start.elapsed().as_millis() as u64);
                            full_response.push_str(&token);
                            sentence_buffer.push_str(&token);
                            let _ = self.app_handle.emit("llm_token", serde_json::json!({ "token": token }));

                            if token.contains('.') || token.contains('?') || token.contains('!') {
                                let tts_text = sentence_buffer.trim().to_string();
                                sentence_buffer.clear();
                                if !tts_text.is_empty() {
                                    let tts_start = std::time::Instant::now();
                                    match tts_router.synthesize(tts_text, lang, online, mode).await {
                                        Ok(audio) => {
                                            if first_audio_ms.is_none() {
                                                first_audio_ms = Some(tts_start.elapsed().as_millis() as u64);
                                            }
                                            if let Some(pipeline) = self.audio_pipeline.write().as_mut() {
                                                pipeline.play(&audio);
                                            }
                                        }
                                        Err(e) => log::error!("TTS error: {}", e),
                                    }
                                }
                            }
                        }
                        
                        while let Some(token) = stream.next().await {
                            if first_token_ms.is_none() {
                                first_token_ms = Some(llm_start.elapsed().as_millis() as u64);
                            }
                            full_response.push_str(&token);
                            sentence_buffer.push_str(&token);
                            
                            let _ = self.app_handle.emit("llm_token", serde_json::json!({ "token": token }));

                            // Basic sentence boundary detection
                            if token.contains('.') || token.contains('?') || token.contains('!') {
                                // 3. TTS for sentence
                                let tts_text = sentence_buffer.trim().to_string();
                                sentence_buffer.clear();
                                
                                if !tts_text.is_empty() {
                                    let tts_start = std::time::Instant::now();
                                    match tts_router.synthesize(tts_text, lang, online, mode).await {
                                        Ok(audio) => {
                                            if first_audio_ms.is_none() {
                                                first_audio_ms = Some(tts_start.elapsed().as_millis() as u64);
                                            }
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
                            let tts_start = std::time::Instant::now();
                            if let Ok(audio) = tts_router.synthesize(tts_text, lang, online, mode).await {
                                if first_audio_ms.is_none() {
                                    first_audio_ms = Some(tts_start.elapsed().as_millis() as u64);
                                }
                                if let Some(pipeline) = self.audio_pipeline.write().as_mut() {
                                    pipeline.play(&audio);
                                }
                            }
                        }
                        
                        let total_ms = turn_start.elapsed().as_millis() as u64;
                        let _ = self.app_handle.emit("turn_latency", serde_json::json!({
                            "stt_ms": stt_done_ms,
                            "llm_first_token_ms": first_token_ms.unwrap_or(0),
                            "tts_first_audio_ms": first_audio_ms.unwrap_or(0),
                            "total_turn_ms": total_ms,
                            "transcript": transcript,
                            "response": full_response
                        }));

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
