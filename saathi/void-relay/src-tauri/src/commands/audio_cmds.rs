use serde::{Deserialize, Serialize};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub kind: String, // "input" or "output"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEndpointDiagnostic {
    pub name: String,
    pub flow: String, // "input" or "output"
    pub is_default: bool,
    pub default_sample_rate: u32,
    pub channels: u16,
    pub sample_format: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPlaybackResult {
    pub duration_sec: f32,
    pub samples_count: usize,
    pub max_rms: f32,
    pub status: String,
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

    devices
}

#[tauri::command]
pub fn get_audio_diagnostics() -> Vec<AudioEndpointDiagnostic> {
    let host = cpal::default_host();
    let mut diagnostics = Vec::new();

    let def_in_name = host.default_input_device().and_then(|d| d.name().ok());
    let def_out_name = host.default_output_device().and_then(|d| d.name().ok());

    if let Ok(inputs) = host.input_devices() {
        for d in inputs {
            if let Ok(name) = d.name() {
                let is_default = def_in_name.as_ref().map(|n| n == &name).unwrap_or(false);
                let (sr, ch, fmt) = match d.default_input_config() {
                    Ok(cfg) => (cfg.sample_rate().0, cfg.channels(), format!("{:?}", cfg.sample_format())),
                    Err(_) => (0, 0, "Unknown".to_string()),
                };
                diagnostics.push(AudioEndpointDiagnostic {
                    name,
                    flow: "input".to_string(),
                    is_default,
                    default_sample_rate: sr,
                    channels: ch,
                    sample_format: fmt,
                    state: "Active".to_string(),
                });
            }
        }
    }

    if let Ok(outputs) = host.output_devices() {
        for d in outputs {
            if let Ok(name) = d.name() {
                let is_default = def_out_name.as_ref().map(|n| n == &name).unwrap_or(false);
                let (sr, ch, fmt) = match d.default_output_config() {
                    Ok(cfg) => (cfg.sample_rate().0, cfg.channels(), format!("{:?}", cfg.sample_format())),
                    Err(_) => (0, 0, "Unknown".to_string()),
                };
                diagnostics.push(AudioEndpointDiagnostic {
                    name,
                    flow: "output".to_string(),
                    is_default,
                    default_sample_rate: sr,
                    channels: ch,
                    sample_format: fmt,
                    state: "Active".to_string(),
                });
            }
        }
    }

    diagnostics
}

fn find_output_device(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if name.is_empty() || name == "default" {
        return host.default_output_device();
    }
    if let Ok(devices) = host.output_devices() {
        for d in devices {
            if let Ok(n) = d.name() {
                if n.to_lowercase().contains(&name.to_lowercase()) {
                    return Some(d);
                }
            }
        }
    }
    host.default_output_device()
}

fn find_input_device(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if name.is_empty() || name == "default" {
        return host.default_input_device();
    }
    if let Ok(devices) = host.input_devices() {
        for d in devices {
            if let Ok(n) = d.name() {
                if n.to_lowercase().contains(&name.to_lowercase()) {
                    return Some(d);
                }
            }
        }
    }
    host.default_input_device()
}

#[tauri::command]
pub async fn play_test_tone(
    output_device: String,
    frequency: Option<f32>,
    duration_ms: Option<u64>,
) -> Result<String, String> {
    let freq = frequency.unwrap_or(1000.0);
    let dur = duration_ms.unwrap_or(1500);

    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        let res = (|| -> Result<String, String> {
            let device = find_output_device(&output_device)
                .ok_or_else(|| "Selected output device not found".to_string())?;

            let dev_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            let config = device.default_output_config()
                .map_err(|e| format!("Failed to get output config: {}", e))?;
            let sample_rate = config.sample_rate().0 as f32;
            let channels = config.channels() as usize;

            let sample_clock = Arc::new(Mutex::new(0f32));
            let clock_clone = sample_clock.clone();

            let err_fn = |err| log::error!("Tone stream error: {}", err);

            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    device.build_output_stream(
                        &config.into(),
                        move |data: &mut [f32], _: &_| {
                            let mut clock = clock_clone.lock();
                            for frame in data.chunks_mut(channels) {
                                let sample = (*clock * freq * 2.0 * std::f32::consts::PI / sample_rate).sin() * 0.25;
                                for s in frame.iter_mut() {
                                    *s = sample;
                                }
                                *clock += 1.0;
                            }
                        },
                        err_fn,
                        None,
                    ).map_err(|e| format!("Failed to build F32 stream: {}", e))?
                }
                cpal::SampleFormat::I16 => {
                    device.build_output_stream(
                        &config.into(),
                        move |data: &mut [i16], _: &_| {
                            let mut clock = clock_clone.lock();
                            for frame in data.chunks_mut(channels) {
                                let sample = ((*clock * freq * 2.0 * std::f32::consts::PI / sample_rate).sin() * 0.25 * 32767.0) as i16;
                                for s in frame.iter_mut() {
                                    *s = sample;
                                }
                                *clock += 1.0;
                            }
                        },
                        err_fn,
                        None,
                    ).map_err(|e| format!("Failed to build I16 stream: {}", e))?
                }
                _ => return Err("Unsupported audio format".to_string()),
            };

            stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
            std::thread::sleep(std::time::Duration::from_millis(dur));
            drop(stream);

            Ok(format!("Successfully played {}Hz tone for {}ms on '{}'", freq, dur, dev_name))
        })();

        let _ = tx.send(res);
    });

    rx.await.map_err(|_| "Test tone thread panicked".to_string())?
}

#[tauri::command]
pub async fn record_and_playback_test(
    input_device: String,
    output_device: String,
    duration_secs: Option<u64>,
) -> Result<RecordPlaybackResult, String> {
    let dur = duration_secs.unwrap_or(5);
    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        let res = (|| -> Result<RecordPlaybackResult, String> {
            let in_device = find_input_device(&input_device)
                .ok_or_else(|| "Selected input device not found".to_string())?;
            let in_name = in_device.name().unwrap_or_else(|_| "Default Input".to_string());

            let in_cfg = in_device.default_input_config()
                .map_err(|e| format!("Input config error: {}", e))?;
            let in_sr = in_cfg.sample_rate().0;
            let in_ch = in_cfg.channels() as usize;

            let buffer = Arc::new(Mutex::new(Vec::<i16>::new()));
            let buf_rec = buffer.clone();
            let is_recording = Arc::new(AtomicBool::new(true));
            let rec_flag = is_recording.clone();

            let err_fn = |err| log::error!("Capture error: {}", err);

            let in_stream = match in_cfg.sample_format() {
                cpal::SampleFormat::F32 => in_device.build_input_stream(
                    &in_cfg.into(),
                    move |data: &[f32], _: &_| {
                        if rec_flag.load(Ordering::Relaxed) {
                            let mut b = buf_rec.lock();
                            for frame in data.chunks(in_ch) {
                                let sample = (frame[0].clamp(-1.0, 1.0) * 32767.0) as i16;
                                b.push(sample);
                            }
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| format!("Failed to build input stream: {}", e))?,
                cpal::SampleFormat::I16 => in_device.build_input_stream(
                    &in_cfg.into(),
                    move |data: &[i16], _: &_| {
                        if rec_flag.load(Ordering::Relaxed) {
                            let mut b = buf_rec.lock();
                            for frame in data.chunks(in_ch) {
                                b.push(frame[0]);
                            }
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| format!("Failed to build input stream: {}", e))?,
                _ => return Err("Unsupported input sample format".to_string()),
            };

            in_stream.play().map_err(|e| format!("Failed to start capture: {}", e))?;
            std::thread::sleep(std::time::Duration::from_secs(dur));
            is_recording.store(false, Ordering::SeqCst);
            drop(in_stream);

            let recorded_samples = {
                let b = buffer.lock();
                b.clone()
            };

            if recorded_samples.is_empty() {
                return Err("No audio samples captured from device".to_string());
            }

            // Calculate RMS
            let mut sum_sq: f64 = 0.0;
            for &s in &recorded_samples {
                let norm = s as f64 / 32768.0;
                sum_sq += norm * norm;
            }
            let rms = (sum_sq / recorded_samples.len() as f64).sqrt() as f32;

            // Playback to output_device
            let out_device = find_output_device(&output_device)
                .ok_or_else(|| "Selected output device not found for playback".to_string())?;
            let out_name = out_device.name().unwrap_or_else(|_| "Default Output".to_string());

            let out_cfg = out_device.default_output_config()
                .map_err(|e| format!("Output config error: {}", e))?;
            let out_sr = out_cfg.sample_rate().0;
            let out_ch = out_cfg.channels() as usize;

            let playback_samples = if in_sr != out_sr {
                crate::audio::playback_resample(&recorded_samples, in_sr, out_sr)
            } else {
                recorded_samples.clone()
            };

            let play_idx = Arc::new(Mutex::new(0usize));
            let p_idx = play_idx.clone();
            let play_buf = Arc::new(playback_samples);
            let p_buf = play_buf.clone();

            let out_stream = match out_cfg.sample_format() {
                cpal::SampleFormat::F32 => out_device.build_output_stream(
                    &out_cfg.into(),
                    move |data: &mut [f32], _: &_| {
                        let mut idx = p_idx.lock();
                        for frame in data.chunks_mut(out_ch) {
                            let sample = if *idx < p_buf.len() {
                                let s = p_buf[*idx] as f32 / 32768.0;
                                *idx += 1;
                                s
                            } else {
                                0.0
                            };
                            for out_sample in frame.iter_mut() {
                                *out_sample = sample;
                            }
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| format!("Failed to build playback stream: {}", e))?,
                cpal::SampleFormat::I16 => out_device.build_output_stream(
                    &out_cfg.into(),
                    move |data: &mut [i16], _: &_| {
                        let mut idx = p_idx.lock();
                        for frame in data.chunks_mut(out_ch) {
                            let sample = if *idx < p_buf.len() {
                                let s = p_buf[*idx];
                                *idx += 1;
                                s
                            } else {
                                0
                            };
                            for out_sample in frame.iter_mut() {
                                *out_sample = sample;
                            }
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| format!("Failed to build playback stream: {}", e))?,
                _ => return Err("Unsupported output sample format".to_string()),
            };

            out_stream.play().map_err(|e| format!("Failed to start playback: {}", e))?;
            std::thread::sleep(std::time::Duration::from_secs(dur));
            drop(out_stream);

            Ok(RecordPlaybackResult {
                duration_sec: dur as f32,
                samples_count: recorded_samples.len(),
                max_rms: rms,
                status: format!(
                    "Captured {}s (RMS: {:.4}) from '{}' and played back to '{}'",
                    dur, rms, in_name, out_name
                ),
            })
        })();

        let _ = tx.send(res);
    });

    rx.await.map_err(|_| "Record/Playback thread panicked".to_string())?
}

#[tauri::command]
pub async fn run_bt_diagnostics_dump() -> Result<String, String> {
    let output = std::process::Command::new("powershell")
        .args(&[
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            "c:\\Users\\Puns0\\Documents\\SAATHI\\saathi\\scripts\\dump_audio_bt.ps1"
        ])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(stdout)
            } else {
                Err(format!("Diagnostics failed: {}\n{}", stderr, stdout))
            }
        }
        Err(e) => Err(format!("Failed to run dump script: {}", e)),
    }
}

#[tauri::command]
pub async fn auto_route_audio() -> Result<String, String> {
    log::info!("Tauri command: auto_route_audio triggered");
    let output = std::process::Command::new("powershell")
        .args(&[
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            "c:\\Users\\Puns0\\Documents\\SAATHI\\saathi\\scripts\\dump_audio_bt.ps1"
        ])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(stdout)
            } else {
                Err(format!("Auto-routing failed: {}\n{}", stderr, stdout))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
