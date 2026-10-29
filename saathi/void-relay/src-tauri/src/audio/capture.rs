use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use tokio::sync::mpsc;

pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    pub running: Arc<AtomicBool>,
}

unsafe impl Send for AudioCapture {}
unsafe impl Sync for AudioCapture {}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, device: &cpal::Device, tx: mpsc::Sender<Vec<i16>>) -> anyhow::Result<()> {
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        log::info!("Starting audio capture: {} Hz, {} channels", sample_rate, channels);

        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let (mut prod, mut cons) = HeapRb::<i16>::new(sample_rate as usize * 2).split();

        // Background task to send buffers from ringbuffer to channel
        let run_flag = running.clone();
        tokio::spawn(async move {
            let chunk_size = sample_rate as usize / 10; // 100ms chunks
            let mut chunk = Vec::with_capacity(chunk_size);
            while run_flag.load(Ordering::SeqCst) {
                while let Some(sample) = cons.pop() {
                    chunk.push(sample);
                    if chunk.len() >= chunk_size {
                        let resampled = resample(&chunk, sample_rate, 8000);
                        if tx.send(resampled).await.is_err() {
                            run_flag.store(false, Ordering::SeqCst);
                            break;
                        }
                        chunk.clear();
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        let err_fn = move |err| {
            log::error!("an error occurred on the input audio stream: {}", err);
        };

        let channels = config.channels() as usize;
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    if running.load(Ordering::SeqCst) {
                        let mut i = 0;
                        while i < data.len() {
                            let sample = data[i];
                            let _ = prod.push(sample);
                            i += channels;
                        }
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if running.load(Ordering::SeqCst) {
                        let mut i = 0;
                        while i < data.len() {
                            let sample = (data[i] * 32768.0) as i16;
                            let _ = prod.push(sample);
                            i += channels;
                        }
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

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.stream = None;
    }
}

fn resample(input: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
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
