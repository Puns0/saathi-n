use std::sync::Arc;
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;

use crate::models::model_manager::ModelManager;

pub struct PiperTts {
    model_manager: Arc<ModelManager>,
}

impl PiperTts {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }

    pub async fn synthesize(&self, text: &str, model_id: &str) -> anyhow::Result<Vec<i16>> {
        let model_info = self.model_manager.find_model(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model {} not found", model_id))?;
            
        let path = self.model_manager.get_model_path(model_info);
        if !path.exists() {
            anyhow::bail!("Piper Model file not downloaded");
        }

        let piper_exe = if cfg!(windows) { "piper.exe" } else { "piper" };
        
        // Ensure piper is in PATH or specify absolute path in a real impl
        // Here we assume it's available in PATH
        let mut child = Command::new(piper_exe)
            .args([
                "--model", &path.to_string_lossy(),
                "--output_raw" // Outputs raw PCM
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await?;
            stdin.flush().await?;
        }

        let output = child.wait_with_output().await?;
        
        if !output.status.success() {
            anyhow::bail!("Piper execution failed");
        }

        // Piper outputs raw PCM i16 (usually 22050Hz, need to check config)
        // Here we parse raw bytes to i16
        let mut samples = Vec::with_capacity(output.stdout.len() / 2);
        for chunk in output.stdout.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            samples.push(sample);
        }

        // NOTE: In a complete implementation, we'd resample from Piper's native rate to 8000Hz here.
        // Simplified for this scaffolding.

        Ok(samples)
    }
}
