use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // AI Settings
    pub ai_mode: String, // "online", "offline", "auto"
    pub groq_api_key: String,
    pub sarvam_api_key: String,
    pub default_language: String, // "auto", "en", "hi", "kn"
    pub system_prompt: String,
    pub vad_sensitivity: f32,
    pub silence_timeout_ms: u32,
    pub streaming_tts: bool,
    pub user_name: String,

    // Phone Settings
    pub android_phone_name: String,
    pub ios_phone_name: String,
    pub ws_port: u16,
    pub preferred_phone: String, // "auto", "android", "ios"
    pub connection_timeout_sec: u32,

    // BT Settings
    pub saved_bt_devices: Vec<SavedBtDevice>,
    pub bt_auto_reconnect: bool,
    pub bt_audio_device_override: Option<String>,

    // Audio Device Selection (empty string = use system default)
    pub audio_input_device: String,
    pub audio_output_device: String,

    // Notification Settings
    pub notify_call_started: bool,
    pub notify_call_ended: bool,
    pub notify_download_complete: bool,
    pub notify_connection_lost: bool,

    // System
    pub launch_on_startup: bool,
    pub log_level: String, // "debug", "info", "warn", "error"

    // Telegram
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,

    // Active Model Config
    pub active_stt_english: String,
    pub active_stt_indic: String,
    pub active_tts_english: String,
    pub active_tts_hindi: String,
    pub active_tts_kannada: String,
    pub active_llm: String,

    // Runtime (not persisted)
    #[serde(skip)]
    pub online_available: bool,
    #[serde(skip)]
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedBtDevice {
    pub name: String,
    pub address: String,
    pub last_connected: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ai_mode: "online".to_string(),
            groq_api_key: std::env::var("GROQ_API_KEY").unwrap_or_default(),
            sarvam_api_key: std::env::var("SARVAM_API_KEY").unwrap_or_default(),
            default_language: "auto".to_string(),
            system_prompt: "You are a voice assistant making/receiving a phone call on behalf of the user. Rules:\n- Keep ALL responses under 2 sentences unless asked for detail\n- Speak naturally — you are in a live phone call\n- Match the language of the caller (Kannada/Hindi/English)\n- Be polite, professional, and concise\n- Do not reveal you are an AI unless directly asked".to_string(),
            vad_sensitivity: 0.008,
            silence_timeout_ms: 800,
            streaming_tts: true,
            user_name: "User".to_string(),
            android_phone_name: "Android Phone".to_string(),
            ios_phone_name: "iPhone".to_string(),
            ws_port: 7331,
            preferred_phone: "auto".to_string(),
            connection_timeout_sec: 30,
            saved_bt_devices: vec![],
            bt_auto_reconnect: true,
            bt_audio_device_override: None,
            audio_input_device: String::new(),
            audio_output_device: String::new(),
            notify_call_started: true,
            notify_call_ended: true,
            notify_download_complete: true,
            notify_connection_lost: true,
            launch_on_startup: false,
            log_level: "info".to_string(),
            telegram_bot_token: String::new(),
            telegram_chat_id: String::new(),
            active_stt_english: "whisper-base".to_string(),
            active_stt_indic: "indicwhisper-medium".to_string(),
            active_tts_english: "piper-en_IN-medium".to_string(),
            active_tts_hindi: "ai4bharat-indic-tts-hindi".to_string(),
            active_tts_kannada: "ai4bharat-indic-tts-kannada".to_string(),
            active_llm: "llama3.2:3b".to_string(),
            online_available: false,
            data_dir: PathBuf::new(),
        }
    }
}

impl AppSettings {
    pub fn load(data_dir: &Path) -> Self {
        let config_path = data_dir.join("settings.json");
        let mut settings = if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        };
        settings.data_dir = data_dir.to_path_buf();
        settings
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = self.data_dir.join("settings.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, json)?;
        Ok(())
    }
}
