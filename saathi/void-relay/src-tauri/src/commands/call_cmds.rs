use tauri::State;
use crate::AppState;
use crate::call::history::CallRecord;

#[tauri::command]
pub async fn initiate_call(state: State<'_, AppState>, number: String, platform_hint: Option<String>) -> Result<String, String> {
    state.orchestrator.initiate_call(number, platform_hint).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hangup_call(state: State<'_, AppState>) -> Result<(), String> {
    state.orchestrator.hangup().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_call_state(state: State<'_, AppState>) -> String {
    format!("{:?}", state.orchestrator.get_state())
}

#[tauri::command]
pub fn get_call_history(state: State<'_, AppState>, limit: usize) -> Result<Vec<CallRecord>, String> {
    crate::call::history::get_history(&state.db, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_call_history(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let history = crate::call::history::get_history(&state.db, 1000).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_call_history(state: State<'_, AppState>) -> Result<(), String> {
    crate::call::history::clear_history(&state.db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn say_text(state: State<'_, AppState>, text: String) -> Result<(), String> {
    state.orchestrator.say_text(text).await.map_err(|e| e.to_string())
}
