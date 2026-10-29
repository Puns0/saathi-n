use tauri::State;
use crate::AppState;
use std::fs;

#[tauri::command]
pub fn get_logs(state: State<'_, AppState>, lines: usize) -> Result<Vec<String>, String> {
    let s = state.settings.read();
    let log_path = s.data_dir.join("voidrelay.log");
    
    if !log_path.exists() {
        return Ok(vec!["No logs found".to_string()]);
    }
    
    let content = fs::read_to_string(log_path).map_err(|e| e.to_string())?;
    let all_lines: Vec<&str> = content.lines().collect();
    
    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };
    
    Ok(all_lines[start..].iter().map(|&s| s.to_string()).collect())
}

#[tauri::command]
pub fn export_logs(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let s = state.settings.read();
    let log_path = s.data_dir.join("voidrelay.log");
    if log_path.exists() {
        fs::copy(log_path, path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_connected_phones(state: State<'_, AppState>) -> Vec<String> {
    state.ws_server.get_connected_platforms()
}
