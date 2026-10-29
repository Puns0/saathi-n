use cpal::Device;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HfpState {
    Disconnected,
    Connecting,
    Connected,
    Active, // Audio routing active
    Error,
}

pub struct HfpAudioDevices {
    pub input: Device,
    pub output: Device,
    pub sample_rate: u32,
    pub channels: u16,
}

#[async_trait::async_trait]
pub trait HfpManager: Send + Sync {
    async fn scan_devices(&self) -> anyhow::Result<Vec<BluetoothDevice>>;
    async fn connect_hfp(&self, address: &str) -> anyhow::Result<()>;
    async fn disconnect(&self) -> anyhow::Result<()>;
    fn get_audio_devices(&self) -> Option<HfpAudioDevices>;
    fn is_hfp_active(&self) -> bool;
    fn on_state_change(&self, cb: Box<dyn Fn(HfpState) + Send + Sync>);
}

pub fn create_hfp_manager() -> Box<dyn HfpManager> {
    #[cfg(target_os = "windows")]
    {
        Box::new(super::platform::windows::WindowsHfpManager::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(super::platform::macos::MacOsHfpManager::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(super::platform::linux::LinuxHfpManager::new())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        unimplemented!("Unsupported platform for HFP")
    }
}
