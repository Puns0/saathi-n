use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use cpal::traits::{DeviceTrait, HostTrait};

use super::capture::AudioCapture;
use super::playback::AudioPlayback;
use super::vad::{Vad, VadState};

pub struct AudioPipeline {
    capture: AudioCapture,
    playback: AudioPlayback,
    pub suppress_capture: Arc<AtomicBool>,
}

/// Resolve a named audio device from cpal, or fall back to the system default.
fn resolve_input_device(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if name.is_empty() {
        return host.default_input_device();
    }
    if let Ok(devices) = host.input_devices() {
        for d in devices {
            if d.name().map(|n| n.to_lowercase().contains(&name.to_lowercase())).unwrap_or(false) {
                log::info!("Using configured input device: {}", d.name().unwrap_or_default());
                return Some(d);
            }
        }
    }
    log::warn!("Configured input device '{}' not found, falling back to default", name);
    host.default_input_device()
}

fn resolve_output_device(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if name.is_empty() {
        return host.default_output_device();
    }
    if let Ok(devices) = host.output_devices() {
        for d in devices {
            if d.name().map(|n| n.to_lowercase().contains(&name.to_lowercase())).unwrap_or(false) {
                log::info!("Using configured output device: {}", d.name().unwrap_or_default());
                return Some(d);
            }
        }
    }
    log::warn!("Configured output device '{}' not found, falling back to default", name);
    host.default_output_device()
}

impl AudioPipeline {
    pub fn new() -> Self {
        Self {
            capture: AudioCapture::new(),
            playback: AudioPlayback::new(),
            suppress_capture: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the pipeline using explicitly provided cpal devices (e.g. from HFP).
    pub fn start_with_devices(
        &mut self,
        input: &cpal::Device,
        output: &cpal::Device,
        tx_speech: mpsc::Sender<Vec<i16>>,
        vad: Vad,
    ) -> anyhow::Result<()> {
        self.start_inner(input, output, tx_speech, vad)
    }

    /// Start the pipeline using device names from settings.
    /// Empty string = use system default. "CABLE Output" = VB-Cable input, etc.
    pub fn start_with_names(
        &mut self,
        input_name: &str,
        output_name: &str,
        tx_speech: mpsc::Sender<Vec<i16>>,
        vad: Vad,
    ) -> anyhow::Result<()> {
        let input = resolve_input_device(input_name)
            .ok_or_else(|| anyhow::anyhow!("No input audio device available"))?;
        let output = resolve_output_device(output_name)
            .ok_or_else(|| anyhow::anyhow!("No output audio device available"))?;

        log::info!(
            "Audio pipeline: IN='{}' OUT='{}'",
            input.name().unwrap_or_default(),
            output.name().unwrap_or_default()
        );

        self.start_inner(&input, &output, tx_speech, vad)
    }

    fn start_inner(
        &mut self,
        input: &cpal::Device,
        output: &cpal::Device,
        tx_speech: mpsc::Sender<Vec<i16>>,
        mut vad: Vad,
    ) -> anyhow::Result<()> {
        log::info!("Starting audio pipeline...");

        let (tx_capture, mut rx_capture) = mpsc::channel(100);

        self.capture.start(input, tx_capture)?;
        self.playback.start(output)?;

        let suppress = self.suppress_capture.clone();

        let mut current_speech = Vec::new();
        let mut pre_roll: std::collections::VecDeque<Vec<i16>> = std::collections::VecDeque::new();

        tokio::spawn(async move {
            while let Some(chunk) = rx_capture.recv().await {
                if suppress.load(Ordering::SeqCst) {
                    pre_roll.clear();
                    continue; // skip capture while AI is speaking (prevents echo)
                }
                let state = vad.process_chunk(&chunk);

                match state {
                    VadState::Speaking => {
                        // Prepend pre-roll if we just started speaking
                        if current_speech.is_empty() {
                            while let Some(pre_chunk) = pre_roll.pop_front() {
                                current_speech.extend_from_slice(&pre_chunk);
                            }
                        }
                        current_speech.extend_from_slice(&chunk);
                    }
                    VadState::SpeechComplete => {
                        if !current_speech.is_empty() {
                            let _ = tx_speech.send(current_speech.clone()).await;
                            current_speech.clear();
                        }
                        pre_roll.clear();
                    }
                    VadState::Silence => {
                        pre_roll.push_back(chunk);
                        if pre_roll.len() > 3 {
                            pre_roll.pop_front();
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn play(&mut self, audio: &[i16]) {
        self.suppress_capture.store(true, Ordering::SeqCst);
        self.playback.play_audio(audio);
        let frames = audio.len();
        let ms = frames as u64 * 1000 / 8000; // Assuming 8kHz
        let supp = self.suppress_capture.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(ms + 200)).await;
            supp.store(false, Ordering::SeqCst);
        });
    }

    pub fn stop(&mut self) {
        self.capture.stop();
        self.playback.stop();
    }
}
