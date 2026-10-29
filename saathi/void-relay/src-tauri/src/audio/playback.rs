use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapRb, SharedRb};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AudioPlayback {
    stream: Option<cpal::Stream>,
    pub running: Arc<AtomicBool>,
    producer: Option<ringbuf::Producer<i16, Arc<SharedRb<i16, Vec<std::mem::MaybeUninit<i16>>>>>>,
    pub sample_rate: u32,
}

unsafe impl Send for AudioPlayback {}
unsafe impl Sync for AudioPlayback {}

impl AudioPlayback {
    pub fn new() -> Self {
        Self {
            stream: None,
            running: Arc::new(AtomicBool::new(false)),
            producer: None,
            sample_rate: 8000,
        }
    }

    pub fn start(&mut self, device: &cpal::Device) -> anyhow::Result<()> {
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        self.sample_rate = sample_rate;
        
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        // 5 seconds buffer
        let (prod, mut cons) = HeapRb::<i16>::new(sample_rate as usize * 5).split();
        self.producer = Some(prod);

        let err_fn = |err| log::error!("an error occurred on the output audio stream: {}", err);

        let channels = config.channels() as usize;
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config.into(),
                move |data: &mut [i16], _: &_| {
                    let mut i = 0;
                    while i < data.len() {
                        let sample = cons.pop().unwrap_or(0);
                        for c in 0..channels {
                            if i + c < data.len() {
                                data[i + c] = sample;
                            }
                        }
                        i += channels;
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &_| {
                    let mut i = 0;
                    while i < data.len() {
                        let sample = cons.pop().unwrap_or(0) as f32 / 32768.0;
                        for c in 0..channels {
                            if i + c < data.len() {
                                data[i + c] = sample;
                            }
                        }
                        i += channels;
                    }
                },
                err_fn,
                None,
            )?,
            _ => anyhow::bail!("Unsupported sample format"),
        };

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn play_audio(&mut self, audio: &[i16]) {
        if let Some(prod) = &mut self.producer {
            let resampled = resample(audio, 8000, self.sample_rate);
            for &sample in &resampled {
                // If buffer is full, we might drop samples. In a real system we'd wait/block.
                let _ = prod.push(sample);
            }
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.stream = None;
        self.producer = None;
    }
}

pub fn resample(input: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
    if from_rate == to_rate {
        return input.to_vec();
    }
    let ratio = to_rate as f64 / from_rate as f64;
    let output_len = (input.len() as f64 * ratio).round() as usize;
    let mut output = Vec::with_capacity(output_len);
    for i in 0..output_len {
        let pos = i as f64 / ratio;
        let idx = pos.floor() as usize;
        let frac = pos - idx as f64;
        if idx + 1 < input.len() {
            let s0 = input[idx] as f64;
            let s1 = input[idx + 1] as f64;
            let sample = s0 + (s1 - s0) * frac;
            output.push(sample.round() as i16);
        } else if idx < input.len() {
            output.push(input[idx]);
        }
    }
    output
}

