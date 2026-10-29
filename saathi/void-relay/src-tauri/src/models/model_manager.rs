use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use sha2::{Sha256, Digest};
use crate::models::downloader::DownloadManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelCategory { STT, TTS, LLM }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String, pub name: String, pub category: ModelCategory,
    pub language: Vec<String>, pub size_mb: u64, pub ram_required_mb: u64,
    pub download_url: String, pub filename: String, pub sha256: String,
    pub quantization: String, pub min_ram_gb: f32,
    pub quality_score: u8, pub speed_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedModel {
    pub info: ModelInfo, pub local_path: String,
    pub downloaded_at: String, pub verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSlot { SttEnglish, SttIndic, TtsEnglish, TtsHindi, TtsKannada, Llm }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveModelConfig {
    pub stt_english: Option<String>, pub stt_indic: Option<String>,
    pub tts_english: Option<String>, pub tts_hindi: Option<String>,
    pub tts_kannada: Option<String>, pub llm: Option<String>,
}

pub struct ModelManager {
    pub models_dir: PathBuf,
    pub registry: Vec<ModelInfo>,
    pub downloaded: Arc<RwLock<HashMap<String, DownloadedModel>>>,
    pub download_manager: Arc<DownloadManager>,
}

fn mi(id: &str, name: &str, cat: ModelCategory, lang: Vec<&str>, size: u64, ram: u64, url: &str, fname: &str, quant: &str, min_ram: f32, q: u8, s: u8) -> ModelInfo {
    ModelInfo { id: id.into(), name: name.into(), category: cat, language: lang.iter().map(|l| l.to_string()).collect(), size_mb: size, ram_required_mb: ram, download_url: url.into(), filename: fname.into(), sha256: String::new(), quantization: quant.into(), min_ram_gb: min_ram, quality_score: q, speed_score: s }
}

impl ModelManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let models_dir = data_dir.join("models");
        for d in &["stt/whisper","stt/indicwhisper","tts/piper","tts/indic_tts","llm"] {
            std::fs::create_dir_all(models_dir.join(d)).ok();
        }
        let registry = vec![
            mi("whisper-tiny","Whisper Tiny",ModelCategory::STT,vec!["en","all"],75,75,"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin","ggml-tiny.bin","none",1.0,4,10),
            mi("whisper-base","Whisper Base",ModelCategory::STT,vec!["en","all"],148,150,"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin","ggml-base.bin","none",1.0,5,9),
            mi("whisper-small","Whisper Small",ModelCategory::STT,vec!["en","all"],488,500,"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin","ggml-small.bin","none",2.0,7,7),
            mi("whisper-medium","Whisper Medium",ModelCategory::STT,vec!["en","all"],1533,1500,"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin","ggml-medium.bin","none",4.0,8,5),
            mi("whisper-large-v3-turbo","Whisper Large V3 Turbo",ModelCategory::STT,vec!["en","all"],1600,2100,"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin","ggml-large-v3-turbo.bin","none",4.0,10,6),
            mi("faster-whisper-large-v3-turbo-int8","Faster Whisper V3 Turbo INT8",ModelCategory::STT,vec!["en","all"],700,700,"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin","ggml-large-v3-turbo-q8_0.bin","int8",2.0,9,8),
            mi("indicwhisper-medium","IndicWhisper Medium",ModelCategory::STT,vec!["hi","kn"],1000,1000,"https://huggingface.co/ai4bharat/indicwhisper/resolve/main/ggml-medium.bin","indicwhisper-ggml-medium.bin","none",2.0,8,6),
            mi("indicwhisper-large","IndicWhisper Large",ModelCategory::STT,vec!["hi","kn"],2000,2000,"https://huggingface.co/ai4bharat/indicwhisper/resolve/main/ggml-large.bin","indicwhisper-ggml-large.bin","none",4.0,10,4),
            mi("piper-en_IN-low","Piper EN-IN Low",ModelCategory::TTS,vec!["en"],30,50,"https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_IN/cmu_indic_fem/low/en_IN-cmu_indic_fem-low.onnx","en_IN-low.onnx","none",1.0,5,10),
            mi("piper-en_IN-medium","Piper EN-IN Medium",ModelCategory::TTS,vec!["en"],60,100,"https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_IN/cmu_indic_fem/medium/en_IN-cmu_indic_fem-medium.onnx","en_IN-medium.onnx","none",1.0,7,8),
            mi("piper-en_IN-high","Piper EN-IN High",ModelCategory::TTS,vec!["en"],200,300,"https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_IN/cmu_indic_fem/high/en_IN-cmu_indic_fem-high.onnx","en_IN-high.onnx","none",1.0,9,5),
            mi("ai4bharat-indic-tts-hindi","AI4Bharat TTS Hindi",ModelCategory::TTS,vec!["hi"],300,400,"https://huggingface.co/ai4bharat/indic-tts/resolve/main/hindi.onnx","indic-tts-hindi.onnx","none",1.0,9,6),
            mi("ai4bharat-indic-tts-kannada","AI4Bharat TTS Kannada",ModelCategory::TTS,vec!["kn"],300,400,"https://huggingface.co/ai4bharat/indic-tts/resolve/main/kannada.onnx","indic-tts-kannada.onnx","none",1.0,9,6),
            mi("llama3.2:1b","Llama 3.2 1B",ModelCategory::LLM,vec!["all"],800,800,"ollama://llama3.2:1b","llama3.2:1b","q4",2.0,4,10),
            mi("llama3.2:3b","Llama 3.2 3B",ModelCategory::LLM,vec!["all"],2000,2000,"ollama://llama3.2:3b","llama3.2:3b","q4",4.0,6,8),
            mi("mistral:7b-instruct-q4","Mistral 7B Q4",ModelCategory::LLM,vec!["all"],4100,4100,"ollama://mistral:7b-instruct-q4_0","mistral:7b-instruct-q4_0","q4",6.0,9,5),
            mi("mistral:7b-instruct-q8","Mistral 7B Q8",ModelCategory::LLM,vec!["all"],7000,7000,"ollama://mistral:7b-instruct-q8_0","mistral:7b-instruct-q8_0","q8",10.0,10,3),
            mi("llama3.1:8b-instruct-q4","Llama 3.1 8B Q4",ModelCategory::LLM,vec!["all"],4500,4500,"ollama://llama3.1:8b-instruct-q4_0","llama3.1:8b-instruct-q4_0","q4",6.0,8,5),
        ];
        let downloaded = Arc::new(RwLock::new(HashMap::new()));
        let download_manager = Arc::new(DownloadManager::new(models_dir.clone()));
        let mgr = Self { models_dir, registry, downloaded, download_manager };
        mgr.scan_downloaded();
        mgr
    }

    pub fn scan_downloaded(&self) {
        let mut dl = self.downloaded.write();
        dl.clear();
        for model in &self.registry {
            if model.category == ModelCategory::LLM { continue; }
            let path = self.get_model_path(model);
            if path.exists() {
                dl.insert(model.id.clone(), DownloadedModel {
                    info: model.clone(), local_path: path.to_string_lossy().into(),
                    downloaded_at: chrono::Utc::now().to_rfc3339(), verified: true,
                });
            }
        }
        if let Ok(out) = std::process::Command::new("ollama").arg("list").output() {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                for model in &self.registry {
                    if model.category == ModelCategory::LLM && s.contains(&model.filename) {
                        dl.insert(model.id.clone(), DownloadedModel {
                            info: model.clone(), local_path: format!("ollama://{}",model.filename),
                            downloaded_at: chrono::Utc::now().to_rfc3339(), verified: true,
                        });
                    }
                }
            }
        }
    }

    pub fn get_model_path(&self, model: &ModelInfo) -> PathBuf {
        match model.category {
            ModelCategory::STT if model.id.starts_with("indic") => self.models_dir.join("stt/indicwhisper").join(&model.filename),
            ModelCategory::STT => self.models_dir.join("stt/whisper").join(&model.filename),
            ModelCategory::TTS if model.id.starts_with("piper") => self.models_dir.join("tts/piper").join(&model.filename),
            ModelCategory::TTS => self.models_dir.join("tts/indic_tts").join(&model.filename),
            ModelCategory::LLM => self.models_dir.join("llm").join(&model.filename),
        }
    }

    pub fn find_model(&self, id: &str) -> Option<&ModelInfo> { self.registry.iter().find(|m| m.id == id) }
    pub fn is_downloaded(&self, id: &str) -> bool { self.downloaded.read().contains_key(id) }

    pub fn delete_model(&self, id: &str) -> anyhow::Result<()> {
        if let Some(model) = self.find_model(id) {
            if model.category == ModelCategory::LLM {
                std::process::Command::new("ollama").args(["rm",&model.filename]).output()?;
            } else {
                let p = self.get_model_path(model);
                if p.exists() { std::fs::remove_file(p)?; }
            }
            self.downloaded.write().remove(id);
        }
        Ok(())
    }

    pub fn verify_integrity(&self, id: &str) -> bool {
        if let Some(model) = self.find_model(id) {
            if model.category == ModelCategory::LLM { return self.is_downloaded(id); }
            let p = self.get_model_path(model);
            if !p.exists() { return false; }
            if model.sha256.is_empty() { return true; }
            if let Ok(bytes) = std::fs::read(&p) {
                return format!("{:x}", Sha256::digest(&bytes)) == model.sha256;
            }
        }
        false
    }
}
