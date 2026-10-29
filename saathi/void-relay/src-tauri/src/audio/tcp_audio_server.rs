use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use parking_lot::RwLock;
use once_cell::sync::Lazy;

/// TCP audio server for bidirectional PCM streaming between iPhone and Mac.
///
/// Architecture:
///   - Port 7332: Mac listens, iPhone connects → receives call audio from iPhone
///   - Port 7333: iPhone listens, Mac connects → sends TTS audio to iPhone
///
/// Audio format:
///   - 8000 Hz, mono, 16-bit signed integer (i16), little-endian
///   - Framing: 4-byte LE length prefix + raw PCM bytes
///   - Chunk size: 160 samples = 20ms per packet
///   - Bitrate: ~128 kbps
pub struct TcpAudioServer {
    receive_port: u16,   // 7332 — receive iPhone audio
    send_port: u16,      // 7333 — send TTS to iPhone
    audio_tx: mpsc::Sender<Vec<i16>>,      // to VAD/STT pipeline
    tts_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Vec<i16>>>>, // from TTS
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

/// Global storage for the iPhone's IP address, set when iPhone connects via WebSocket.
static IPHONE_IP: Lazy<RwLock<Option<String>>> =
    Lazy::new(|| RwLock::new(None));

/// Tracks whether the TCP receive server has an active connection.
static TCP_RECEIVE_CONNECTED: Lazy<std::sync::atomic::AtomicBool> =
    Lazy::new(|| std::sync::atomic::AtomicBool::new(false));

/// Tracks whether the TCP send connection is active.
static TCP_SEND_CONNECTED: Lazy<std::sync::atomic::AtomicBool> =
    Lazy::new(|| std::sync::atomic::AtomicBool::new(false));

/// Tracks bytes received per second (for UI stats).
static RECV_BYTES_PER_SEC: Lazy<std::sync::atomic::AtomicU64> =
    Lazy::new(|| std::sync::atomic::AtomicU64::new(0));

/// Tracks bytes sent per second (for UI stats).
static SEND_BYTES_PER_SEC: Lazy<std::sync::atomic::AtomicU64> =
    Lazy::new(|| std::sync::atomic::AtomicU64::new(0));

impl TcpAudioServer {
    /// Create a new TCP audio server.
    ///
    /// `audio_tx` — channel sender for PCM chunks going to the VAD/STT pipeline
    /// `tts_rx`   — channel receiver for TTS PCM chunks to send to iPhone
    pub fn new(
        audio_tx: mpsc::Sender<Vec<i16>>,
        tts_rx: mpsc::Receiver<Vec<i16>>,
    ) -> Self {
        Self {
            receive_port: 7332,
            send_port: 7333,
            audio_tx,
            tts_rx: Arc::new(tokio::sync::Mutex::new(tts_rx)),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start the TCP audio server.
    ///
    /// Spawns two tasks:
    /// 1. Receive server on port 7332 — accepts iPhone connections, forwards PCM to pipeline
    /// 2. Send loop — connects to iPhone:7333, sends TTS PCM back
    pub async fn start(&self) -> anyhow::Result<()> {
        self.is_running.store(true, std::sync::atomic::Ordering::SeqCst);

        // ── Receive server (Mac listens on 7332, iPhone connects) ──
        let receive_listener = TcpListener::bind(
            format!("0.0.0.0:{}", self.receive_port)
        ).await?;
        log::info!("TCP audio receive server listening on 0.0.0.0:{}", self.receive_port);

        let audio_tx = self.audio_tx.clone();
        let is_running = self.is_running.clone();
        tokio::spawn(async move {
            loop {
                if !is_running.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                match receive_listener.accept().await {
                    Ok((socket, addr)) => {
                        log::info!("iPhone audio connected from {}", addr);
                        TCP_RECEIVE_CONNECTED.store(true, std::sync::atomic::Ordering::SeqCst);

                        // Store iPhone IP for the send connection
                        {
                            let mut ip = IPHONE_IP.write();
                            *ip = Some(addr.ip().to_string());
                        }

                        let tx = audio_tx.clone();
                        tokio::spawn(async move {
                            Self::handle_receive(socket, tx).await;
                            TCP_RECEIVE_CONNECTED.store(false, std::sync::atomic::Ordering::SeqCst);
                            log::info!("iPhone audio stream disconnected");
                        });
                    }
                    Err(e) => {
                        log::error!("TCP accept error: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        // ── Send loop (Mac connects to iPhone:7333) ──
        let tts_rx = self.tts_rx.clone();
        let is_running2 = self.is_running.clone();
        tokio::spawn(async move {
            Self::send_loop(tts_rx, is_running2).await;
        });

        // ── Stats reset loop ──
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                RECV_BYTES_PER_SEC.store(0, std::sync::atomic::Ordering::SeqCst);
                SEND_BYTES_PER_SEC.store(0, std::sync::atomic::Ordering::SeqCst);
            }
        });

        Ok(())
    }

    /// Receive PCM from iPhone → forward to audio pipeline.
    ///
    /// Protocol: [4-byte LE u32 length] [PCM payload bytes]
    async fn handle_receive(
        mut socket: TcpStream,
        tx: mpsc::Sender<Vec<i16>>,
    ) {
        loop {
            // Read 4-byte length prefix
            let mut len_buf = [0u8; 4];
            if socket.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            let length = u32::from_le_bytes(len_buf) as usize;

            // Sanity check — max 32KB per packet (2 seconds at 8kHz mono 16-bit)
            if length > 32000 || length == 0 {
                log::warn!("Invalid TCP audio packet length: {}", length);
                break;
            }

            // Read PCM payload
            let mut pcm_buf = vec![0u8; length];
            if socket.read_exact(&mut pcm_buf).await.is_err() {
                break;
            }

            // Convert bytes → i16 samples (little-endian)
            let samples: Vec<i16> = pcm_buf
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect();

            RECV_BYTES_PER_SEC.fetch_add(length as u64, std::sync::atomic::Ordering::Relaxed);

            let _ = tx.send(samples).await;
        }
    }

    /// Send TTS PCM to iPhone via TCP.
    ///
    /// Waits for the iPhone IP to be known (from WebSocket hello or TCP receive connection),
    /// then connects to iPhone:7333 and streams TTS audio.
    async fn send_loop(
        tts_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Vec<i16>>>>,
        is_running: Arc<std::sync::atomic::AtomicBool>,
    ) {
        loop {
            if !is_running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;

            let iphone_ip = {
                let ip = IPHONE_IP.read();
                ip.clone()
            };

            if let Some(ip) = iphone_ip {
                let addr = format!("{}:7333", ip);
                log::info!("Attempting to connect to iPhone TTS receiver at {}", addr);

                match TcpStream::connect(&addr).await {
                    Ok(mut stream) => {
                        log::info!("Connected to iPhone TTS receiver at {}", addr);
                        TCP_SEND_CONNECTED.store(true, std::sync::atomic::Ordering::SeqCst);

                        let mut rx = tts_rx.lock().await;
                        loop {
                            match rx.recv().await {
                                Some(samples) => {
                                    // Convert i16 → bytes (little-endian)
                                    let bytes: Vec<u8> = samples
                                        .iter()
                                        .flat_map(|s| s.to_le_bytes())
                                        .collect();

                                    // Length-prefix + send
                                    let len = (bytes.len() as u32).to_le_bytes();
                                    let mut packet = Vec::with_capacity(4 + bytes.len());
                                    packet.extend_from_slice(&len);
                                    packet.extend_from_slice(&bytes);

                                    SEND_BYTES_PER_SEC.fetch_add(
                                        bytes.len() as u64,
                                        std::sync::atomic::Ordering::Relaxed,
                                    );

                                    if stream.write_all(&packet).await.is_err() {
                                        log::warn!("Lost connection to iPhone TTS receiver");
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }

                        TCP_SEND_CONNECTED.store(false, std::sync::atomic::Ordering::SeqCst);
                    }
                    Err(_) => {
                        // iPhone not ready yet, retry
                        continue;
                    }
                }
            }
        }
    }

    /// Called when iPhone connects via WebSocket — store iPhone IP for TTS send connection.
    pub fn set_iphone_ip(ip: String) {
        log::info!("iPhone IP set to: {}", ip);
        let mut stored = IPHONE_IP.write();
        *stored = Some(ip);
    }

    /// Get the current iPhone IP if known.
    pub fn get_iphone_ip() -> Option<String> {
        IPHONE_IP.read().clone()
    }

    /// Check if the receive connection (iPhone → Mac) is active.
    pub fn is_receive_connected() -> bool {
        TCP_RECEIVE_CONNECTED.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Check if the send connection (Mac → iPhone) is active.
    pub fn is_send_connected() -> bool {
        TCP_SEND_CONNECTED.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get receive bytes per second (for UI stats).
    pub fn get_recv_bps() -> u64 {
        RECV_BYTES_PER_SEC.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get send bytes per second (for UI stats).
    pub fn get_send_bps() -> u64 {
        SEND_BYTES_PER_SEC.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Stop the TCP audio server.
    pub fn stop(&self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
        TCP_RECEIVE_CONNECTED.store(false, std::sync::atomic::Ordering::SeqCst);
        TCP_SEND_CONNECTED.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
