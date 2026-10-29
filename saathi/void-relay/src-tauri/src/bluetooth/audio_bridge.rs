use std::sync::Arc;
use crate::bluetooth::hfp_manager::{HfpManager, HfpAudioDevices, create_hfp_manager};

pub struct AudioBridge {
    manager: Box<dyn HfpManager>,
}

impl AudioBridge {
    pub fn new() -> Self {
        Self {
            manager: create_hfp_manager(),
        }
    }

    pub async fn connect(&self, address: &str) -> anyhow::Result<()> {
        self.manager.connect_hfp(address).await
    }

    pub async fn disconnect(&self) -> anyhow::Result<()> {
        self.manager.disconnect().await
    }

    pub fn get_devices(&self) -> Option<HfpAudioDevices> {
        self.manager.get_audio_devices()
    }
}
