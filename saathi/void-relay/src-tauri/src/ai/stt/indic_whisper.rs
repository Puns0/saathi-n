use std::sync::Arc;
use crate::models::model_manager::ModelManager;

pub struct IndicWhisperLocal {
    _model_manager: Arc<ModelManager>,
}

impl IndicWhisperLocal {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { _model_manager: model_manager }
    }

    pub async fn transcribe(&self, _audio: &[i16], _model_id: &str) -> anyhow::Result<String> {
        anyhow::bail!("Local IndicWhisper STT is not available in online mode. Use Sarvam STT instead.")
    }
}
