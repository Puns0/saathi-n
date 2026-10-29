use std::sync::Arc;
use parking_lot::RwLock;
use reqwest::Client;
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::config::settings::AppSettings;

pub struct SarvamStt {
    settings: Arc<RwLock<AppSettings>>,
    client: Client,
}

impl SarvamStt {
    pub fn new(settings: Arc<RwLock<AppSettings>>) -> Self {
        Self {
            settings,
            client: Client::new(),
        }
    }

    pub async fn transcribe(&self, audio: &[i16]) -> anyhow::Result<String> {
        let api_key = {
            let s = self.settings.read();
            s.sarvam_api_key.clone()
        };

        if api_key.is_empty() {
            anyhow::bail!("Sarvam API key not set");
        }

        // Convert i16 PCM to minimal 16 kHz WAV in memory
        let resampled = crate::audio::playback_resample(audio, 8000, 16000);
        let mut wav_data = Vec::new();
        {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: 16000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut wav_data), spec)?;
            for &sample in &resampled {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }

        let file_part = reqwest::multipart::Part::bytes(wav_data)
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", "saaras:v3") // Used latest model
            .text("language_code", "auto-detect")
            .text("with_diarization", "false");

        let response = self.client.post("https://api.sarvam.ai/speech-to-text-translate")
            .header("api-subscription-key", api_key)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            anyhow::bail!("Sarvam API error: {}", err);
        }

        let json: serde_json::Value = response.json().await?;
        let transcript = json["transcript"].as_str().unwrap_or_default().to_string();

        Ok(transcript)
    }
}
