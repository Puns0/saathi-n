use serde::{Deserialize, Serialize};
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub kind: String, // "input" or "output"
}

#[tauri::command]
pub fn list_audio_devices() -> Vec<AudioDeviceInfo> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    // List input devices
    if let Ok(inputs) = host.input_devices() {
        for d in inputs {
            if let Ok(name) = d.name() {
                devices.push(AudioDeviceInfo { name, kind: "input".to_string() });
            }
        }
    }

    // List output devices
    if let Ok(outputs) = host.output_devices() {
        for d in outputs {
            if let Ok(name) = d.name() {
                devices.push(AudioDeviceInfo { name, kind: "output".to_string() });
            }
        }
    }

    // Log them for debugging
    log::info!("Available audio devices ({} total):", devices.len());
    for d in &devices {
        log::info!("  [{}] {}", d.kind, d.name);
    }

    devices
}

#[tauri::command]
pub async fn auto_route_audio() -> Result<String, String> {
    log::info!("Tauri command: auto_route_audio triggered");
    
    let output = std::process::Command::new("powershell")
        .args(&[
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            "d:\\Projects\\SAATHI\\saathi\\scratch\\auto_route.ps1"
        ])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            log::info!("auto_route stdout: {}", stdout);
            if out.status.success() {
                Ok(stdout)
            } else {
                log::error!("auto_route failed: {}", stderr);
                Err(format!("Auto-routing failed: {}\n{}", stderr, stdout))
            }
        }
        Err(e) => {
            log::error!("Failed to execute powershell: {}", e);
            Err(e.to_string())
        }
    }
}
