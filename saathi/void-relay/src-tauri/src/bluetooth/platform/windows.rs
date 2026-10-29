use crate::bluetooth::hfp_manager::{BluetoothDevice, HfpAudioDevices, HfpManager, HfpState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

pub struct WindowsHfpManager {
    state: Arc<RwLock<HfpState>>,
    active: Arc<AtomicBool>,
}

impl WindowsHfpManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HfpState::Disconnected)),
            active: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl HfpManager for WindowsHfpManager {
    async fn scan_devices(&self) -> anyhow::Result<Vec<BluetoothDevice>> {
        log::info!("Scanning Windows BT devices (stub)");
        Ok(vec![])
    }

    async fn connect_hfp(&self, _address: &str) -> anyhow::Result<()> {
        log::info!("Connecting Windows HFP (stub)");
        *self.state.write() = HfpState::Connected;
        self.active.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&self) -> anyhow::Result<()> {
        log::info!("Disconnecting Windows HFP (stub)");
        *self.state.write() = HfpState::Disconnected;
        self.active.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn get_audio_devices(&self) -> Option<HfpAudioDevices> {
        // Find Windows Hands-Free AG Audio devices
        use cpal::traits::{DeviceTrait, HostTrait};
        let host = cpal::default_host();
        
        let mut input_dev = None;
        let mut output_dev = None;

        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    let lower_name = name.to_lowercase();
                    if lower_name.contains("hands-free") || lower_name.contains("handsfree") {
                        input_dev = Some(device);
                        break;
                    }
                }
            }
        }

        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    let lower_name = name.to_lowercase();
                    if lower_name.contains("hands-free") || lower_name.contains("handsfree") {
                        output_dev = Some(device);
                        break;
                    }
                }
            }
        }

        let input = input_dev.or_else(|| host.default_input_device())?;
        let output = output_dev.or_else(|| host.default_output_device())?;

        Some(HfpAudioDevices {
            input,
            output,
            sample_rate: 8000,
            channels: 1,
        })
    }

    fn is_hfp_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    fn on_state_change(&self, _cb: Box<dyn Fn(HfpState) + Send + Sync>) {
    }
}
