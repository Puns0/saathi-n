use std::sync::Arc;
use parking_lot::RwLock;

use crate::ai::language_detector::LanguageCode;
use crate::config::settings::AppSettings;
use crate::models::model_manager::ModelManager;
use crate::ai::engine_router::AiMode;
use super::{sarvam_stt::SarvamStt, whisper_local::WhisperLocal, indic_whisper::IndicWhisperLocal};

pub struct SttRouter {
    settings: Arc<RwLock<AppSettings>>,
    model_manager: Arc<ModelManager>,
    sarvam: SarvamStt,
    whisper: WhisperLocal,
    indic_whisper: IndicWhisperLocal,
}

impl SttRouter {
    pub fn new(settings: Arc<RwLock<AppSettings>>, model_manager: Arc<ModelManager>) -> Self {
        Self {
            settings: settings.clone(),
            model_manager: model_manager.clone(),
            sarvam: SarvamStt::new(settings.clone()),
            whisper: WhisperLocal::new(model_manager.clone()),
            indic_whisper: IndicWhisperLocal::new(model_manager.clone()),
        }
    }

    pub async fn transcribe(&self, audio: &[i16], hint: LanguageCode, online: bool, mode: AiMode) -> anyhow::Result<String> {
        let is_online = match mode {
            AiMode::Online => true,
            AiMode::Offline => false,
            AiMode::Auto => online,
        };

        if is_online {
            match hint {
                LanguageCode::Kannada | LanguageCode::Hindi | LanguageCode::Mixed => {
                    log::debug!("Routing STT to Sarvam (Online, Indic/Mixed)");
                    match self.sarvam.transcribe(audio).await {
                        Ok(text) => return Ok(text),
                        Err(e) => log::warn!("Sarvam STT failed: {}, falling back to local", e),
                    }
                }
                LanguageCode::English => {
                    // Ideally Groq STT here, but we'll use Sarvam as a fallback for everything online
                    // for simplicity unless explicitly handling Groq API
                    log::debug!("Routing STT to Sarvam (Online, English)");
                    match self.sarvam.transcribe(audio).await {
                        Ok(text) => return Ok(text),
                        Err(e) => log::warn!("Sarvam STT failed: {}, falling back to local", e),
                    }
                }
            }
        }

        // Offline / Fallback routing
        match hint {
            LanguageCode::Kannada | LanguageCode::Hindi | LanguageCode::Mixed => {
                log::debug!("Routing STT to IndicWhisper (Offline)");
                let model_id = {
                    let s = self.settings.read();
                    s.active_stt_indic.clone()
                };
                self.indic_whisper.transcribe(audio, &model_id).await
            }
            LanguageCode::English => {
                log::debug!("Routing STT to Whisper (Offline)");
                let model_id = {
                    let s = self.settings.read();
                    s.active_stt_english.clone()
                };
                self.whisper.transcribe(audio, &model_id).await
            }
        }
    }
}
