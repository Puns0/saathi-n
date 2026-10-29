use std::sync::Arc;
use parking_lot::RwLock;
use reqwest::Client;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use hound::WavReader;
use std::io::Cursor;

use crate::ai::language_detector::LanguageCode;
use crate::config::settings::AppSettings;

pub struct SarvamTts {
    settings: Arc<RwLock<AppSettings>>,
    client: Client,
}

impl SarvamTts {
    pub fn new(settings: Arc<RwLock<AppSettings>>) -> Self {
        Self {
            settings,
            client: Client::new(),
        }
    }

    pub async fn synthesize(&self, text: &str, lang: LanguageCode) -> anyhow::Result<Vec<i16>> {
        let api_key = {
            let s = self.settings.read();
            s.sarvam_api_key.clone()
        };

        if api_key.is_empty() {
            anyhow::bail!("Sarvam API key not set");
        }

        let target_lang = match lang {
            LanguageCode::Hindi | LanguageCode::Mixed => "hi-IN",
            LanguageCode::Kannada => "kn-IN",
            LanguageCode::English => "en-IN",
            _ => "hi-IN", // Fallback
        };

        let speaker = match lang {
            LanguageCode::Hindi | LanguageCode::Mixed => "shubh",
            LanguageCode::Kannada => "kavya",
            LanguageCode::English => "shubh",
            _ => "shubh",
        };

        let body = serde_json::json!({
            "inputs": [text],
            "target_language_code": target_lang,
            "speaker": speaker,
            "pace": 1.0,
            "speech_sample_rate": 8000,
            "enable_preprocessing": true,
            "model": "bulbul:v3"
        });

        let response = self.client.post("https://api.sarvam.ai/text-to-speech")
            .header("api-subscription-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            anyhow::bail!("Sarvam API error: {}", err);
        }

        let json: serde_json::Value = response.json().await?;
        let audios = json["audios"].as_array().ok_or_else(|| anyhow::anyhow!("No audio returned"))?;
        
        let base64_audio = audios[0].as_str().ok_or_else(|| anyhow::anyhow!("Invalid audio format"))?;
        let wav_data = STANDARD.decode(base64_audio)?;

        // Decode WAV to PCM i16
        let mut reader = WavReader::new(Cursor::new(wav_data))?;
        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap_or(0)).collect();

        Ok(samples)
    }
}
