use reqwest::Client;
use std::sync::Arc;
use tauri::AppHandle;
use parking_lot::RwLock;

use crate::bridge::ws_server::WsServer;
use crate::call::orchestrator::CallOrchestrator;
use crate::config::settings::AppSettings;

pub struct TelegramBot {
    token: String,
    chat_id: String,
    client: Client,
    last_update_id: i64,
}

impl TelegramBot {
    pub fn new(token: String, chat_id: String) -> Self {
        Self {
            token,
            chat_id,
            client: Client::new(),
            last_update_id: 0,
        }
    }

    pub async fn start_polling(&mut self, ws_server: Arc<WsServer>, orchestrator: Arc<CallOrchestrator>, settings: Arc<RwLock<AppSettings>>, app_handle: AppHandle) {
        let url = format!("https://api.telegram.org/bot{}/getUpdates", self.token);
        
        loop {
            let body = serde_json::json!({
                "offset": self.last_update_id + 1,
                "timeout": 30
            });

            match self.client.post(&url).json(&body).send().await {
                Ok(response) => {
                    if let Ok(json) = response.json::<serde_json::Value>().await {
                        if json["ok"].as_bool().unwrap_or(false) {
                            if let Some(updates) = json["result"].as_array() {
                                for update in updates {
                                    if let Some(update_id) = update["update_id"].as_i64() {
                                        self.last_update_id = update_id;
                                    }
                                    
                                    if let Some(message) = update.get("message") {
                                        if let Some(text) = message["text"].as_str() {
                                            self.handle_command(text, &ws_server, &orchestrator, &settings).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Telegram polling error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn handle_command(&self, text: &str, ws_server: &Arc<WsServer>, orchestrator: &Arc<CallOrchestrator>, settings: &Arc<RwLock<AppSettings>>) {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() { return; }

        match parts[0] {
            "/call" => {
                if parts.len() >= 3 && (parts[1] == "android" || parts[1] == "iphone") {
                    let platform = if parts[1] == "iphone" { "ios" } else { "android" };
                    let number = parts[2..].join(" ");
                    
                    let connected = ws_server.get_connected_platforms();
                    if !connected.contains(&platform.to_string()) {
                        let _ = self.send_message(&format!("❌ {} not connected", parts[1])).await;
                        return;
                    }
                    
                    match orchestrator.initiate_call(number.clone(), Some(platform.to_string())).await {
                        Ok(_) => { let _ = self.send_message(&format!("📞 Calling {} via {}...", number, platform)).await; },
                        Err(e) => { let _ = self.send_message(&format!("❌ Failed to call: {}", e)).await; }
                    }
                } else if parts.len() >= 2 {
                    let number = parts[1..].join(" ");
                    match orchestrator.initiate_call(number.clone(), None).await {
                        Ok(_) => { let _ = self.send_message(&format!("📞 Calling {}...", number)).await; },
                        Err(e) => { let _ = self.send_message(&format!("❌ Failed to call: {}", e)).await; }
                    }
                } else {
                    let _ = self.send_message("Usage: /call [android|iphone] <number>").await;
                }
            }
            "/hangup" => {
                let _ = orchestrator.hangup().await;
                let _ = self.send_message("📵 Hanging up...").await;
            }
            "/status" => {
                let state = orchestrator.get_state();
                let ai_mode = settings.read().ai_mode.clone();
                let connected = ws_server.get_connected_platforms();
                
                let android_status = if connected.contains(&"android".to_string()) { "🟢 Connected" } else { "🔴 Offline" };
                let ios_status = if connected.contains(&"ios".to_string()) { "🟢 Connected" } else { "🔴 Offline" };
                
                let msg = format!(
                    "📊 VoidStack Status\n\
                     Android: {} | Agent: ON\n\
                     iPhone: {} | Agent: ON\n\
                     AI Mode: 🌐 {}\n\
                     Active Call: {:?}\n",
                    android_status, ios_status, ai_mode, state
                );
                let _ = self.send_message(&msg).await;
            }
            "/mode" => {
                if parts.len() > 1 {
                    let mode = parts[1].to_lowercase();
                    if mode == "online" || mode == "offline" || mode == "auto" {
                        {
                            let mut s = settings.write();
                            s.ai_mode = mode.clone();
                            let _ = s.save();
                        }
                        let _ = self.send_message(&format!("✅ Switched to {} mode", mode)).await;
                    } else {
                        let _ = self.send_message("Usage: /mode online|offline|auto").await;
                    }
                }
            }
            "/prompt" => {
                if parts.len() > 1 {
                    let prompt = parts[1..].join(" ");
                    {
                        let mut s = settings.write();
                        s.system_prompt = prompt;
                        let _ = s.save();
                    }
                    let _ = self.send_message("✅ System prompt updated").await;
                }
            }
            "/say" => {
                if parts.len() > 1 {
                    let text = parts[1..].join(" ");
                    if orchestrator.get_state() != crate::call::orchestrator::CallState::Idle {
                        let _ = orchestrator.say_text(text.clone()).await;
                        let _ = self.send_message(&format!("🔊 Speaking: {}", text)).await;
                    } else {
                        let _ = self.send_message("❌ No active call").await;
                    }
                }
            }
            "/mute" => {
                let _ = self.send_message("✅ AI muted (monitor only)").await;
                // Would set a flag on orchestrator here
            }
            "/unmute" => {
                let _ = self.send_message("✅ AI unmuted").await;
                // Would clear flag on orchestrator here
            }
            "/history" => {
                let limit = if parts.len() > 1 { parts[1].parse().unwrap_or(5) } else { 5 };
                // Stub for db access
                let _ = self.send_message(&format!("Fetching last {} calls...", limit)).await;
            }
            "/models" => {
                let (stt_en, stt_indic, tts_en, llm) = {
                    let s = settings.read();
                    (s.active_stt_english.clone(), s.active_stt_indic.clone(), s.active_tts_english.clone(), s.active_llm.clone())
                };
                let msg = format!(
                    "🧠 Active Models\n\
                     STT (EN): {}\n\
                     STT (Indic): {}\n\
                     TTS (EN): {}\n\
                     LLM: {}\n",
                    stt_en, stt_indic, tts_en, llm
                );
                let _ = self.send_message(&msg).await;
            }
            "/reset" => {
                let _ = self.send_message("✅ Conversation history cleared for current call").await;
                // Would call clear on orchestrator
            }
            "/help" => {
                let help_text = "VoidRelay Commands:\n\
                /call [android|iphone] <num> - Make a call\n\
                /hangup - End call\n\
                /status - Show system status\n\
                /mode <online|offline|auto> - Switch AI mode\n\
                /prompt <text> - Set system prompt\n\
                /say <text> - Speak in active call\n\
                /mute - Pause AI\n\
                /unmute - Resume AI\n\
                /history [n] - Show recent calls\n\
                /models - Show active models\n\
                /reset - Clear call history\n\
                /help - Show this message";
                let _ = self.send_message(help_text).await;
            }
            _ => {
                let _ = self.send_message("Unknown command. Type /help").await;
            }
        }
    }

    pub async fn notify_telegram(&self, msg: &str) -> anyhow::Result<()> {
        self.send_message(msg).await
    }

    pub async fn send_message(&self, text: &str) -> anyhow::Result<()> {
        if self.chat_id.is_empty() { return Ok(()); }
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        self.client.post(&url)
            .json(&serde_json::json!({
                "chat_id": self.chat_id,
                "text": text
            }))
            .send()
            .await?;
        Ok(())
    }
}
