use tauri::{State, AppHandle};
use crate::AppState;
use crate::models::model_manager::{ModelInfo, DownloadedModel, ActiveModelConfig, ModelSlot};
use crate::models::model_recommender::ModelRecommendation;

#[tauri::command]
pub fn list_available_models(state: State<'_, AppState>) -> Vec<ModelInfo> {
    state.model_manager.registry.clone()
}

#[tauri::command]
pub fn list_downloaded_models(state: State<'_, AppState>) -> Vec<DownloadedModel> {
    state.model_manager.downloaded.read().values().cloned().collect()
}

#[tauri::command]
pub async fn download_model(
    state: State<'_, AppState>, 
    app: AppHandle, 
    model_id: String
) -> Result<(), String> {
    let model = state.model_manager.find_model(&model_id)
        .ok_or_else(|| format!("Model {} not found", model_id))?;
        
    let url = model.download_url.clone();
    let path = state.model_manager.get_model_path(model);
    
    state.model_manager.download_manager.download_model(model_id, url, path, app)
        .await
        .map_err(|e| e.to_string())?;
        
    state.model_manager.scan_downloaded();
    Ok(())
}

#[tauri::command]
pub fn cancel_download(state: State<'_, AppState>, model_id: String) {
    state.model_manager.download_manager.cancel_download(&model_id);
}

#[tauri::command]
pub fn delete_model(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    state.model_manager.delete_model(&model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn switch_model(state: State<'_, AppState>, slot: ModelSlot, model_id: String) -> Result<(), String> {
    let mut settings = state.settings.write();
    match slot {
        ModelSlot::SttEnglish => settings.active_stt_english = model_id,
        ModelSlot::SttIndic => settings.active_stt_indic = model_id,
        ModelSlot::TtsEnglish => settings.active_tts_english = model_id,
        ModelSlot::TtsHindi => settings.active_tts_hindi = model_id,
        ModelSlot::TtsKannada => settings.active_tts_kannada = model_id,
        ModelSlot::Llm => settings.active_llm = model_id,
    }
    settings.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_active_models(state: State<'_, AppState>) -> ActiveModelConfig {
    let s = state.settings.read();
    ActiveModelConfig {
        stt_english: Some(s.active_stt_english.clone()),
        stt_indic: Some(s.active_stt_indic.clone()),
        tts_english: Some(s.active_tts_english.clone()),
        tts_hindi: Some(s.active_tts_hindi.clone()),
        tts_kannada: Some(s.active_tts_kannada.clone()),
        llm: Some(s.active_llm.clone()),
    }
}

#[tauri::command]
pub async fn get_recommendations(state: State<'_, AppState>) -> Result<ModelRecommendation, String> {
    use crate::models::device_profiler::DeviceProfile;
    use crate::models::model_recommender::ModelRecommender;
    
    let profile = DeviceProfile::gather();
    let api_key = state.settings.read().groq_api_key.clone();
    
    let recommender = ModelRecommender::new(api_key);
    recommender.get_recommendations(&profile).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_model_integrity(state: State<'_, AppState>, model_id: String) -> bool {
    state.model_manager.verify_integrity(&model_id)
}

#[tauri::command]
pub fn get_device_profile(state: State<'_, AppState>) -> crate::models::device_profiler::DeviceProfile {
    crate::models::device_profiler::DeviceProfile::gather()
}
