use serde::{Deserialize, Serialize};
use crate::models::device_profiler::DeviceProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub primary_stt_english: String,
    pub primary_stt_indic: String,
    pub primary_tts_english: String,
    pub primary_tts_hindi: String,
    pub primary_tts_kannada: String,
    pub primary_llm: String,
    pub reasoning: String,
    pub warnings: Vec<String>,
    pub total_ram_required_gb: f64,
    pub total_disk_required_gb: f64,
    pub confidence: String,
}

const SYSTEM_PROMPT: &str = r#"You are an AI model selection expert. Given a device hardware profile, recommend the optimal local AI models for a real-time voice agent system. The system needs:
1. STT model (speech-to-text) for English and Indian languages (Kannada, Hindi priority)
2. TTS model (text-to-speech) for the same languages
3. LLM model for conversation (via Ollama)

Consider: available RAM (must leave 2GB for OS), CPU speed, GPU availability, disk space. Prioritize real-time performance — latency matters more than perfect accuracy for phone calls.

Available STT options:
- whisper-tiny (75MB RAM, fast, lower accuracy)
- whisper-base (150MB RAM, fast, decent accuracy)
- whisper-small (500MB RAM, good accuracy)
- whisper-medium (1.5GB RAM, very good accuracy)
- whisper-large-v3-turbo (2.1GB RAM, near-perfect accuracy)
- faster-whisper-large-v3-turbo-int8 (0.7GB RAM, near-perfect, quantized)
- indicwhisper-medium (1.0GB RAM, best for Hindi/Kannada)
- indicwhisper-large (2.0GB RAM, best Indic accuracy)

Available TTS options:
- piper-en_IN-low (30MB, fast, decent quality)
- piper-en_IN-medium (60MB, good quality, recommended)
- piper-en_IN-high (200MB, excellent quality, slower)
- ai4bharat-indic-tts-hindi (300MB, natural Hindi)
- ai4bharat-indic-tts-kannada (300MB, natural Kannada)

Available LLM options (via Ollama):
- llama3.2:1b (800MB RAM, very fast, basic reasoning)
- llama3.2:3b (2.0GB RAM, fast, good reasoning)
- mistral:7b-instruct-q4 (4.1GB RAM, excellent reasoning)
- mistral:7b-instruct-q8 (7.0GB RAM, best offline quality)
- llama3.1:8b-instruct-q4 (4.5GB RAM, very good)

Respond ONLY in this exact JSON format, no preamble:
{
  "primary_stt_english": "model_name",
  "primary_stt_indic": "model_name",
  "primary_tts_english": "model_name",
  "primary_tts_hindi": "model_name",
  "primary_tts_kannada": "model_name",
  "primary_llm": "model_name",
  "reasoning": "brief explanation of choices",
  "warnings": ["any warnings about constraints"],
  "total_ram_required_gb": 0.0,
  "total_disk_required_gb": 0.0,
  "confidence": "high|medium|low"
}"#;

pub struct ModelRecommender {
    groq_api_key: String,
}

impl ModelRecommender {
    pub fn new(groq_api_key: String) -> Self {
        Self { groq_api_key }
    }

    pub async fn get_recommendations(&self, profile: &DeviceProfile) -> anyhow::Result<ModelRecommendation> {
        if self.groq_api_key.is_empty() {
            return self.fallback_recommendation(profile);
        }

        let client = reqwest::Client::new();
        let profile_json = serde_json::to_string_pretty(profile)?;
        let user_msg = format!(
            "My device: {}\nRecommend the best model configuration for real-time voice calls.",
            profile_json
        );

        let body = serde_json::json!({
            "model": "llama-3.3-70b-versatile",
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": user_msg}
            ],
            "temperature": 0.3,
            "max_tokens": 1024,
            "response_format": {"type": "json_object"}
        });

        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.groq_api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            log::warn!("Groq API error {}: {}", status, text);
            return self.fallback_recommendation(profile);
        }

        let resp_json: serde_json::Value = response.json().await?;
        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("{}");

        match serde_json::from_str::<ModelRecommendation>(content) {
            Ok(rec) => Ok(rec),
            Err(e) => {
                log::warn!("Failed to parse Groq recommendation: {}, using fallback", e);
                self.fallback_recommendation(profile)
            }
        }
    }

    fn fallback_recommendation(&self, profile: &DeviceProfile) -> anyhow::Result<ModelRecommendation> {
        let available_ram = profile.available_ram_gb - 2.0;

        let (stt_en, stt_indic, llm, total_ram, total_disk) = if available_ram >= 8.0 {
            ("whisper-large-v3-turbo", "indicwhisper-large", "mistral:7b-instruct-q4", 8.2, 12.0)
        } else if available_ram >= 5.0 {
            ("faster-whisper-large-v3-turbo-int8", "indicwhisper-medium", "llama3.2:3b", 3.7, 5.0)
        } else if available_ram >= 3.0 {
            ("whisper-base", "indicwhisper-medium", "llama3.2:3b", 3.15, 4.0)
        } else {
            ("whisper-tiny", "whisper-tiny", "llama3.2:1b", 0.95, 1.5)
        };

        Ok(ModelRecommendation {
            primary_stt_english: stt_en.to_string(),
            primary_stt_indic: stt_indic.to_string(),
            primary_tts_english: "piper-en_IN-medium".to_string(),
            primary_tts_hindi: "ai4bharat-indic-tts-hindi".to_string(),
            primary_tts_kannada: "ai4bharat-indic-tts-kannada".to_string(),
            primary_llm: llm.to_string(),
            reasoning: format!(
                "Based on {:.1}GB available RAM (after 2GB OS reserve). Selected models fit within memory constraints while maximizing quality.",
                available_ram
            ),
            warnings: if available_ram < 3.0 {
                vec!["Low RAM: performance may be limited. Consider closing other applications.".to_string()]
            } else {
                vec![]
            },
            total_ram_required_gb: total_ram,
            total_disk_required_gb: total_disk,
            confidence: if self.groq_api_key.is_empty() { "medium" } else { "high" }.to_string(),
        })
    }
}
