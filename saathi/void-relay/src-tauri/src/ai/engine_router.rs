use std::sync::Arc;
use parking_lot::RwLock;

use crate::config::settings::AppSettings;
use crate::models::model_manager::ModelManager;
use crate::ai::stt::stt_router::SttRouter;
use crate::ai::llm::llm_router::LlmRouter;
use crate::ai::tts::tts_router::TtsRouter;
use crate::ai::language_detector::{LanguageDetector, LanguageCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiMode {
    Online,
    Offline,
    Auto,
}

impl From<&str> for AiMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "online" => AiMode::Online,
            "offline" => AiMode::Offline,
            _ => AiMode::Auto,
        }
    }
}

pub struct EngineRouter {
    pub stt_router: Arc<SttRouter>,
    pub llm_router: Arc<LlmRouter>,
    pub tts_router: Arc<TtsRouter>,
    pub language_detector: Arc<LanguageDetector>,
    settings: Arc<RwLock<AppSettings>>,
}

impl EngineRouter {
    pub fn new(settings: Arc<RwLock<AppSettings>>, model_manager: Arc<ModelManager>) -> Self {
        let stt_router = Arc::new(SttRouter::new(settings.clone(), model_manager.clone()));
        let llm_router = Arc::new(LlmRouter::new(settings.clone(), model_manager.clone()));
        let tts_router = Arc::new(TtsRouter::new(settings.clone(), model_manager.clone()));
        let language_detector = Arc::new(LanguageDetector::new(model_manager.clone()));

        Self {
            stt_router,
            llm_router,
            tts_router,
            language_detector,
            settings,
        }
    }

    pub fn get_mode(&self) -> (AiMode, bool) {
        let s = self.settings.read();
        (AiMode::from(s.ai_mode.as_str()), s.online_available)
    }

    pub fn is_online_mode(&self) -> bool {
        let (mode, online_available) = self.get_mode();
        match mode {
            AiMode::Online => true,
            AiMode::Offline => false,
            AiMode::Auto => online_available,
        }
    }

    pub async fn detect_language(&self, audio: &[i16]) -> LanguageCode {
        self.language_detector.detect(audio).await
    }
}
