use tauri::State;
use crate::AppState;
use crate::bluetooth::hfp_manager::BluetoothDevice;

#[tauri::command]
pub async fn scan_bluetooth(state: State<'_, AppState>) -> Result<Vec<BluetoothDevice>, String> {
    // Stub implementation
    Ok(vec![])
}

#[tauri::command]
pub async fn connect_bluetooth(state: State<'_, AppState>, address: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn disconnect_bluetooth(state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn get_bluetooth_state(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(false)
}
