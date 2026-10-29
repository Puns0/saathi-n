use std::sync::Arc;
use parking_lot::RwLock;
use reqwest::Client;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures_util::StreamExt;
use bytes::Bytes;

use crate::config::settings::AppSettings;
use crate::ai::llm::llm_router::Message;

pub struct GroqClient {
    settings: Arc<RwLock<AppSettings>>,
    client: Client,
}

impl GroqClient {
    pub fn new(settings: Arc<RwLock<AppSettings>>) -> Self {
        Self {
            settings,
            client: Client::new(),
        }
    }

    pub async fn chat(&self, messages: &[Message]) -> anyhow::Result<String> {
        let api_key = {
            let s = self.settings.read();
            s.groq_api_key.clone()
        };

        if api_key.is_empty() {
            anyhow::bail!("Groq API key not set");
        }

        let body = serde_json::json!({
            "model": "llama-3.3-70b-versatile",
            "messages": messages,
            "temperature": 0.3,
            "stream": false
        });

        let response = self.client.post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Groq API error {}: {}", status, text);
        }

        let json: serde_json::Value = response.json().await?;
        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            Ok(content.to_string())
        } else {
            anyhow::bail!("No content returned from Groq API")
        }
    }

    pub async fn stream_chat(&self, messages: &[Message]) -> anyhow::Result<ReceiverStream<String>> {
        let api_key = {
            let s = self.settings.read();
            s.groq_api_key.clone()
        };

        if api_key.is_empty() {
            anyhow::bail!("Groq API key not set");
        }

        let body = serde_json::json!({
            "model": "llama-3.3-70b-versatile",
            "messages": messages,
            "temperature": 0.5,
            "stream": true
        });

        let response = self.client.post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Groq API error {}: {}", status, text);
        }

        let (tx, rx) = mpsc::channel(100);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut buffer = String::new();
            
            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);
                        
                        // Parse SSE lines
                        while let Some(idx) = buffer.find('\n') {
                            let line = buffer[..idx].to_string();
                            buffer = buffer[idx + 1..].to_string();
                            
                            let line = line.trim();
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    break;
                                }
                                
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                        if !content.is_empty() {
                                            if tx.send(content.to_string()).await.is_err() {
                                                return; // Receiver dropped
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Groq stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}
