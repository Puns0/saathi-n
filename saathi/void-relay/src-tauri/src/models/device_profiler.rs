use serde::{Deserialize, Serialize};
use sysinfo::System;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub os: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub total_ram_gb: f32,
    pub available_ram_gb: f32,
    pub has_gpu: bool,
    pub gpu_model: Option<String>,
    pub gpu_vram_gb: Option<f32>,
    pub is_apple_silicon: bool,
    pub has_avx2: bool,
    pub has_cuda: bool,
    pub has_metal: bool,
    pub disk_free_gb: f32,
    pub cpu_benchmark_score: u32,
}

impl DeviceProfile {
    pub fn gather() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let os = format!("{} {}", System::name().unwrap_or_default(), System::os_version().unwrap_or_default());
        let cpu_model = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
        let cpu_cores = sys.physical_core_count().unwrap_or(1) as u32;
        let cpu_threads = sys.cpus().len() as u32;
        let total_ram_gb = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        let available_ram_gb = sys.available_memory() as f32 / (1024.0 * 1024.0 * 1024.0);

        // Disk space
        let disk_free_gb = {
            let disks = sysinfo::Disks::new_with_refreshed_list();
            disks.list().iter()
                .map(|d| d.available_space() as f32 / (1024.0 * 1024.0 * 1024.0))
                .fold(0.0f32, f32::max)
        };

        // Platform detection
        let is_apple_silicon = cfg!(target_os = "macos") && cfg!(target_arch = "aarch64");
        let has_metal = cfg!(target_os = "macos");
        let has_avx2 = Self::detect_avx2();
        let has_cuda = Self::detect_cuda();

        // GPU detection
        let (has_gpu, gpu_model, gpu_vram_gb) = Self::detect_gpu();

        // CPU benchmark
        let cpu_benchmark_score = Self::run_benchmark(cpu_threads);

        DeviceProfile {
            os,
            cpu_model,
            cpu_cores,
            cpu_threads,
            total_ram_gb,
            available_ram_gb,
            has_gpu,
            gpu_model,
            gpu_vram_gb,
            is_apple_silicon,
            has_avx2,
            has_cuda,
            has_metal,
            disk_free_gb,
            cpu_benchmark_score,
        }
    }

    fn detect_avx2() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx2")
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false
        }
    }

    fn detect_cuda() -> bool {
        // Check for nvidia-smi
        std::process::Command::new("nvidia-smi")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn detect_gpu() -> (bool, Option<String>, Option<f32>) {
        // Try nvidia-smi first
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = stdout.trim().split(", ").collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let vram = parts[1].trim().parse::<f32>().unwrap_or(0.0) / 1024.0;
                    return (true, Some(name), Some(vram));
                }
            }
        }

        // Apple Silicon detection
        #[cfg(target_os = "macos")]
        {
            if cfg!(target_arch = "aarch64") {
                let mut sys = System::new_all();
                sys.refresh_all();
                let cpu = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
                if cpu.contains("Apple") {
                    return (true, Some(cpu), None);
                }
            }
        }

        (false, None, None)
    }

    fn run_benchmark(threads: u32) -> u32 {
        // Single-threaded benchmark: 10M float multiplications
        let start = Instant::now();
        let mut val: f64 = 1.000001;
        for _ in 0..10_000_000 {
            val *= 1.000001;
        }
        let single_ms = start.elapsed().as_millis() as f64;
        // Prevent optimization
        if val < 0.0 { log::trace!("{}", val); }

        // Multi-threaded benchmark
        let start = Instant::now();
        let handles: Vec<_> = (0..threads).map(|_| {
            std::thread::spawn(|| {
                let mut v: f64 = 1.000001;
                for _ in 0..10_000_000 {
                    v *= 1.000001;
                }
                v
            })
        }).collect();
        for h in handles {
            let _ = h.join();
        }
        let multi_ms = start.elapsed().as_millis() as f64;

        // Normalize: lower time = higher score, 0-100 scale
        // Reference: 100ms single-thread = score 50
        let single_score = ((100.0 / (single_ms / 50.0)).min(100.0)) as u32;
        let multi_score = ((100.0 / (multi_ms / (50.0 * threads as f64))).min(100.0)) as u32;
        ((single_score + multi_score) / 2).min(100)
    }

    pub fn save(&self, data_dir: &std::path::Path) -> anyhow::Result<()> {
        let path = data_dir.join("device_profile.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(data_dir: &std::path::Path) -> Option<Self> {
        let path = data_dir.join("device_profile.json");
        if path.exists() {
            let content = std::fs::read_to_string(path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }
}
