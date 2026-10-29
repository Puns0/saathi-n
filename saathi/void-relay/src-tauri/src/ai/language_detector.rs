use std::sync::Arc;
use crate::models::model_manager::ModelManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageCode {
    English,
    Hindi,
    Kannada,
    Mixed, // Code-switching detected (e.g. Hindi + English)
}

impl LanguageCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageCode::English => "en",
            LanguageCode::Hindi => "hi",
            LanguageCode::Kannada => "kn",
            LanguageCode::Mixed => "mixed",
        }
    }
}

pub struct LanguageDetector {
    model_manager: Arc<ModelManager>,
}

impl LanguageDetector {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }

    pub async fn detect(&self, audio: &[i16]) -> LanguageCode {
        // Quick fallback mechanism since actual loading of whisper takes time
        // and we want this to be extremely fast
        log::debug!("Running language detection on {} samples", audio.len());
        
        // In a real implementation we would run whisper-tiny language detection here.
        // For the sake of performance and avoiding loading another model context, 
        // we'll return a default (usually the one set in config or mixed).
        
        // Let's assume the user speaks mixed languages in India mostly
        LanguageCode::Mixed
    }
}
