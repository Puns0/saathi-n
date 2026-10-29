use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::config::settings::AppSettings;
use crate::models::model_manager::ModelManager;
use crate::ai::engine_router::AiMode;
use super::{groq_client::GroqClient, ollama_client::OllamaClient};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub struct LlmRouter {
    settings: Arc<RwLock<AppSettings>>,
    model_manager: Arc<ModelManager>,
    groq: GroqClient,
    ollama: OllamaClient,
}

impl LlmRouter {
    pub fn new(settings: Arc<RwLock<AppSettings>>, model_manager: Arc<ModelManager>) -> Self {
        Self {
            settings: settings.clone(),
            model_manager,
            groq: GroqClient::new(settings.clone()),
            ollama: OllamaClient::new(),
        }
    }

    pub async fn chat(&self, input: String, history: &[Message], online: bool, mode: AiMode) -> anyhow::Result<ReceiverStream<String>> {
        let is_online = match mode {
            AiMode::Online => true,
            AiMode::Offline => false,
            AiMode::Auto => online,
        };

        // Construct full history with system prompt
        let system_prompt = {
            let s = self.settings.read();
            let prompt = s.system_prompt.replace("{datetime}", &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            Message {
                role: "system".to_string(),
                content: prompt,
            }
        };

        let mut full_messages = vec![system_prompt];
        full_messages.extend_from_slice(history);
        full_messages.push(Message {
            role: "user".to_string(),
            content: input,
        });

        if is_online {
            log::debug!("Routing LLM to Groq");
            match self.groq.stream_chat(&full_messages).await {
                Ok(stream) => return Ok(stream),
                Err(e) => log::warn!("Groq LLM failed: {}, falling back to Ollama", e),
            }
        }

        log::debug!("Routing LLM to Ollama");
        let model_id = {
            let s = self.settings.read();
            s.active_llm.clone()
        };
        
        let model_name = self.model_manager.find_model(&model_id)
            .map(|m| m.filename.clone())
            .unwrap_or_else(|| "llama3.2:3b".to_string());

        self.ollama.stream_chat(&model_name, &full_messages).await
    }

    pub async fn summarize(&self, transcript: &str) -> anyhow::Result<String> {
        let system_prompt = Message {
            role: "system".to_string(),
            content: "You are a precise call summarizer. Provide a professional, concise summary of the conversation in 2-3 sentences. Focus strictly on key agreements, appointments, or action items. Do not include introductory text, meta-commentary, or headers. Respond directly with the summary.".to_string(),
        };

        let user_message = Message {
            role: "user".to_string(),
            content: format!("Summarize this conversation:\n\n{}", transcript),
        };

        let messages = vec![system_prompt, user_message];
        
        self.groq.chat(&messages).await
    }
}
