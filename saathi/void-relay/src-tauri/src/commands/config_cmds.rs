use tauri::State;
use crate::AppState;
use crate::config::settings::AppSettings;
use local_ip_address::local_ip;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.read().clone()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    let mut s = state.settings.write();
    let data_dir = s.data_dir.clone();
    *s = settings;
    s.data_dir = data_dir;
    s.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_network_ips() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(my_local_ip) = local_ip() {
        ips.push(my_local_ip.to_string());
    }
    // Could also iterate interfaces here
    ips
}

#[tauri::command]
pub async fn test_groq_key(key: String) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let response = client.get("https://api.groq.com/openai/v1/models")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(response.status().is_success())
}

#[tauri::command]
pub async fn test_sarvam_key(key: String) -> Result<bool, String> {
    // Basic test to see if key is structurally valid or auths
    // Sarvam doesn't have a simple "ping" endpoint, so we might just check length or make a small request
    if key.len() > 10 {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn test_telegram_bot(token: String, chat_id: String) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let response = client.post(format!("https://api.telegram.org/bot{}/sendMessage", token))
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": "VoidStack test message"
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(response.status().is_success())
}
