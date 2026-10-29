use std::sync::Arc;
use parking_lot::RwLock;

use crate::ai::language_detector::LanguageCode;
use crate::config::settings::AppSettings;
use crate::models::model_manager::ModelManager;
use crate::ai::engine_router::AiMode;
use super::{sarvam_tts::SarvamTts, piper_tts::PiperTts, indic_tts::IndicTts};

pub struct TtsRouter {
    settings: Arc<RwLock<AppSettings>>,
    model_manager: Arc<ModelManager>,
    sarvam: SarvamTts,
    piper: PiperTts,
    indic_tts: IndicTts,
}

impl TtsRouter {
    pub fn new(settings: Arc<RwLock<AppSettings>>, model_manager: Arc<ModelManager>) -> Self {
        Self {
            settings: settings.clone(),
            model_manager: model_manager.clone(),
            sarvam: SarvamTts::new(settings.clone()),
            piper: PiperTts::new(model_manager.clone()),
            indic_tts: IndicTts::new(model_manager.clone()),
        }
    }

    pub async fn synthesize(&self, text: String, lang: LanguageCode, online: bool, mode: AiMode) -> anyhow::Result<Vec<i16>> {
        let is_online = match mode {
            AiMode::Online => true,
            AiMode::Offline => false,
            AiMode::Auto => online,
        };

        if is_online {
            match lang {
                LanguageCode::Kannada | LanguageCode::Hindi | LanguageCode::Mixed => {
                    log::debug!("Routing TTS to Sarvam (Online, Indic/Mixed)");
                    match self.sarvam.synthesize(&text, lang).await {
                        Ok(audio) => return Ok(audio),
                        Err(e) => log::warn!("Sarvam TTS failed: {}, falling back to local", e),
                    }
                }
                LanguageCode::English => {
                    log::debug!("Routing TTS to Sarvam (Online, English)");
                    match self.sarvam.synthesize(&text, lang).await {
                        Ok(audio) => return Ok(audio),
                        Err(e) => {
                            log::warn!("Sarvam TTS failed for English: {}, falling back to local Piper", e);
                            let model_id = self.settings.read().active_tts_english.clone();
                            return self.piper.synthesize(&text, &model_id).await;
                        }
                    }
                }
            }
        }

        // Offline / Fallback routing
        match lang {
            LanguageCode::Kannada => {
                log::debug!("Routing TTS to IndicTTS (Offline Kannada)");
                let model_id = self.settings.read().active_tts_kannada.clone();
                self.indic_tts.synthesize(&text, "kn", &model_id).await
            }
            LanguageCode::Hindi | LanguageCode::Mixed => {
                log::debug!("Routing TTS to IndicTTS (Offline Hindi)");
                let model_id = self.settings.read().active_tts_hindi.clone();
                self.indic_tts.synthesize(&text, "hi", &model_id).await
            }
            LanguageCode::English => {
                log::debug!("Routing TTS to Piper (Offline English)");
                let model_id = self.settings.read().active_tts_english.clone();
                self.piper.synthesize(&text, &model_id).await
            }
        }
    }
}
