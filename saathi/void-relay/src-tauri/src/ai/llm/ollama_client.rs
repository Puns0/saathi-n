use reqwest::Client;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures_util::StreamExt;

use crate::ai::llm::llm_router::Message;

pub struct OllamaClient {
    client: Client,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn stream_chat(&self, model: &str, messages: &[Message]) -> anyhow::Result<ReceiverStream<String>> {
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "options": {
                "num_predict": 150,
                "temperature": 0.5
            }
        });

        let response = self.client.post("http://localhost:11434/api/chat")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama error: {}", text);
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
                        
                        // Parse NDJSON lines
                        while let Some(idx) = buffer.find('\n') {
                            let line = buffer[..idx].to_string();
                            buffer = buffer[idx + 1..].to_string();
                            
                            let line = line.trim();
                            if line.is_empty() { continue; }
                            
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                                if let Some(content) = json["message"]["content"].as_str() {
                                    if !content.is_empty() {
                                        if tx.send(content.to_string()).await.is_err() {
                                            return; // Receiver dropped
                                        }
                                    }
                                }
                                if json["done"].as_bool() == Some(true) {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Ollama stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}
