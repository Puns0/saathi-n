use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Vad {
    threshold: f32,
    speech_min_ms: u32,
    silence_timeout_ms: u32,
    sample_rate: u32,
    
    // Internal state
    is_speaking: bool,
    speech_frames: u32,
    silence_frames: u32,
    suppress_capture: Arc<AtomicBool>,
}

impl Vad {
    pub fn new(threshold: f32, speech_min_ms: u32, silence_timeout_ms: u32, sample_rate: u32, suppress_capture: Arc<AtomicBool>) -> Self {
        Self {
            threshold,
            speech_min_ms,
            silence_timeout_ms,
            sample_rate,
            is_speaking: false,
            speech_frames: 0,
            silence_frames: 0,
            suppress_capture,
        }
    }

    pub fn set_sensitivity(&mut self, threshold: f32) {
        self.threshold = threshold;
    }
    
    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.silence_timeout_ms = timeout_ms;
    }

    pub fn process_chunk(&mut self, audio: &[i16]) -> VadState {
        if self.suppress_capture.load(Ordering::Relaxed) {
            self.reset();
            return VadState::Silence;
        }

        let energy = self.calculate_rms(audio);
        let frames_per_ms = self.sample_rate as f32 / 1000.0;
        let chunk_ms = audio.len() as f32 / frames_per_ms;

        if energy > self.threshold {
            self.silence_frames = 0;
            self.speech_frames += chunk_ms as u32;

            if self.speech_frames >= self.speech_min_ms {
                if !self.is_speaking {
                    log::info!("VAD: Speech detected! (RMS: {:.4})", energy);
                }
                self.is_speaking = true;
                return VadState::Speaking;
            }
        } else {
            if self.is_speaking {
                self.silence_frames += chunk_ms as u32;
                if self.silence_frames >= self.silence_timeout_ms {
                    log::info!("VAD: Speech completed (RMS under threshold). Processing...");
                    self.reset();
                    return VadState::SpeechComplete;
                }
                return VadState::Speaking;
            } else {
                self.speech_frames = 0;
            }
        }

        VadState::Silence
    }

    fn calculate_rms(&self, audio: &[i16]) -> f32 {
        if audio.is_empty() { return 0.0; }
        let mut sum_sq = 0.0;
        for &sample in audio {
            let s = sample as f32 / 32768.0;
            sum_sq += s * s;
        }
        (sum_sq / audio.len() as f32).sqrt()
    }

    fn reset(&mut self) {
        self.is_speaking = false;
        self.speech_frames = 0;
        self.silence_frames = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadState {
    Silence,
    Speaking,
    SpeechComplete,
}
