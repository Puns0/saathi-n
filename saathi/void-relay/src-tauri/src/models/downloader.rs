use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Emitter};
use reqwest::Client;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_mb: f64,
    pub total_mb: f64,
    pub speed_mbps: f64,
    pub eta_seconds: u64,
}

pub struct DownloadManager {
    models_dir: PathBuf,
    active_downloads: Arc<RwLock<HashMap<String, Arc<tokio::sync::Notify>>>>,
    client: Client,
}

impl DownloadManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            client: Client::new(),
        }
    }

    pub async fn download_model(
        &self,
        model_id: String,
        url: String,
        target_path: PathBuf,
        app_handle: AppHandle,
    ) -> anyhow::Result<()> {
        // Check if already downloading
        {
            let active = self.active_downloads.read();
            if active.contains_key(&model_id) {
                return Ok(());
            }
        }

        let cancel_notify = Arc::new(tokio::sync::Notify::new());
        {
            let mut active = self.active_downloads.write();
            active.insert(model_id.clone(), cancel_notify.clone());
        }

        let result = self.do_download(model_id.clone(), url, target_path, app_handle, cancel_notify).await;

        // Cleanup
        {
            let mut active = self.active_downloads.write();
            active.remove(&model_id);
        }

        result
    }

    async fn do_download(
        &self,
        model_id: String,
        url: String,
        target_path: PathBuf,
        app_handle: AppHandle,
        cancel_notify: Arc<tokio::sync::Notify>,
    ) -> anyhow::Result<()> {
        if url.starts_with("ollama://") {
            let model_name = url.replace("ollama://", "");
            log::info!("Pulling Ollama model: {}", model_name);
            let mut child = tokio::process::Command::new("ollama")
                .args(["pull", &model_name])
                .spawn()?;
            
            tokio::select! {
                status = child.wait() => {
                    status?;
                    // Send 100% progress
                    let _ = app_handle.emit("download_progress", DownloadProgress {
                        model_id: model_id.clone(),
                        downloaded_mb: 100.0,
                        total_mb: 100.0,
                        speed_mbps: 0.0,
                        eta_seconds: 0,
                    });
                    return Ok(());
                }
                _ = cancel_notify.notified() => {
                    log::info!("Download cancelled for Ollama model: {}", model_id);
                    child.kill().await?;
                    anyhow::bail!("Download cancelled");
                }
            }
        }

        let mut request = self.client.get(&url);

        // Resume capability
        let mut downloaded_bytes = 0u64;
        if target_path.exists() {
            downloaded_bytes = std::fs::metadata(&target_path)?.len();
            if downloaded_bytes > 0 {
                request = request.header("Range", format!("bytes={}-", downloaded_bytes));
            }
        }

        let response = request.send().await?;
        
        let total_size = response.content_length().unwrap_or(0) + downloaded_bytes;
        
        if response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE || 
           (!response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT) {
            // Restart download
            downloaded_bytes = 0;
            std::fs::remove_file(&target_path).ok();
            let response = self.client.get(&url).send().await?;
            if !response.status().is_success() {
                anyhow::bail!("Failed to download: {}", response.status());
            }
        }

        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target_path)
            .await?;
            
        let mut file = tokio::io::BufWriter::new(file);
        let mut stream = response.bytes_stream();

        let mut last_emit = Instant::now();
        let mut downloaded_since_last_emit = 0u64;
        let emit_interval = std::time::Duration::from_millis(100);

        while let Some(chunk) = tokio::select! {
            c = stream.next() => c,
            _ = cancel_notify.notified() => {
                log::info!("Download cancelled for: {}", model_id);
                anyhow::bail!("Download cancelled");
            }
        } {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            
            let chunk_len = chunk.len() as u64;
            downloaded_bytes += chunk_len;
            downloaded_since_last_emit += chunk_len;

            if last_emit.elapsed() >= emit_interval {
                let elapsed_secs = last_emit.elapsed().as_secs_f64();
                let speed_bps = downloaded_since_last_emit as f64 / elapsed_secs;
                let speed_mbps = speed_bps / (1024.0 * 1024.0) * 8.0; // Megabits per sec
                
                let remaining_bytes = total_size.saturating_sub(downloaded_bytes);
                let eta_seconds = if speed_bps > 0.0 {
                    (remaining_bytes as f64 / speed_bps) as u64
                } else {
                    0
                };

                let progress = DownloadProgress {
                    model_id: model_id.clone(),
                    downloaded_mb: downloaded_bytes as f64 / (1024.0 * 1024.0),
                    total_mb: total_size as f64 / (1024.0 * 1024.0),
                    speed_mbps,
                    eta_seconds,
                };

                let _ = app_handle.emit("download_progress", progress);

                last_emit = Instant::now();
                downloaded_since_last_emit = 0;
            }
        }

        file.flush().await?;
        
        // Final progress emit
        let _ = app_handle.emit("download_progress", DownloadProgress {
            model_id: model_id.clone(),
            downloaded_mb: total_size as f64 / (1024.0 * 1024.0),
            total_mb: total_size as f64 / (1024.0 * 1024.0),
            speed_mbps: 0.0,
            eta_seconds: 0,
        });

        Ok(())
    }

    pub fn cancel_download(&self, model_id: &str) {
        if let Some(notify) = self.active_downloads.read().get(model_id) {
            notify.notify_one();
        }
    }
}
