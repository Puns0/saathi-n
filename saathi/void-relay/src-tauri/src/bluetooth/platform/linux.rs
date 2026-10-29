use crate::bluetooth::hfp_manager::{BluetoothDevice, HfpAudioDevices, HfpManager, HfpState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

pub struct LinuxHfpManager {
    state: Arc<RwLock<HfpState>>,
    active: Arc<AtomicBool>,
}

impl LinuxHfpManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HfpState::Disconnected)),
            active: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl HfpManager for LinuxHfpManager {
    async fn scan_devices(&self) -> anyhow::Result<Vec<BluetoothDevice>> {
        log::info!("Scanning Linux BT devices via D-Bus (stub)");
        Ok(vec![])
    }

    async fn connect_hfp(&self, _address: &str) -> anyhow::Result<()> {
        log::info!("Connecting Linux HFP profile (stub)");
        *self.state.write() = HfpState::Connected;
        self.active.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&self) -> anyhow::Result<()> {
        log::info!("Disconnecting Linux HFP (stub)");
        *self.state.write() = HfpState::Disconnected;
        self.active.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn get_audio_devices(&self) -> Option<HfpAudioDevices> {
        // Find PulseAudio/PipeWire handsfree devices
        use cpal::traits::{DeviceTrait, HostTrait};
        let host = cpal::default_host();
        
        let mut input_dev = None;
        let mut output_dev = None;

        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name.contains("handsfree_head_unit") || name.contains("bluez") {
                        input_dev = Some(device);
                        break;
                    }
                }
            }
        }

        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name.contains("handsfree_head_unit") || name.contains("bluez") {
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
