use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use tauri::AppHandle;
use tauri::{Manager, Emitter};

use crate::call::orchestrator::CallOrchestrator;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    #[serde(rename = "hello")]
    Hello { client: String, version: String, device_name: String },
    
    #[serde(rename = "call_started")]
    CallStarted { number: String, call_id: String, client: String },
    
    #[serde(rename = "call_ended")]
    CallEnded { call_id: String, duration_sec: u32, client: String },
    
    #[serde(rename = "bt_connected")]
    BtConnected { device_name: String, client: String },
    
    #[serde(rename = "bt_disconnected")]
    BtDisconnected { client: String },
    
    #[serde(rename = "agent_mode_changed")]
    AgentModeChanged { enabled: bool, client: String },
    
    #[serde(rename = "incoming_call")]
    IncomingCall { number: String, client: String },
    
    #[serde(rename = "heartbeat")]
    Heartbeat { client: String },
    
    #[serde(rename = "heartbeat_ack")]
    HeartbeatAck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsCommand {
    #[serde(rename = "command_call")]
    Call { number: String, call_id: String },
    
    #[serde(rename = "command_hangup")]
    Hangup { call_id: String },
    
    #[serde(rename = "ack")]
    Ack { call_id: String, status: String },
    
    #[serde(rename = "heartbeat")]
    Heartbeat,
}

pub struct ConnectedClient {
    pub id: SocketAddr,
    pub platform: String,
    pub name: String,
    pub sender: mpsc::Sender<Message>,
}

pub struct WsServer {
    clients: Arc<RwLock<HashMap<SocketAddr, ConnectedClient>>>,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(self: Arc<Self>, port: u16, orchestrator: Arc<CallOrchestrator>, app_handle: AppHandle) -> anyhow::Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await?;

        // Heartbeat task
        let clients_ref = self.clients.clone();
        tokio::spawn(async move {
            let ping = Message::Text(serde_json::to_string(&WsCommand::Heartbeat).unwrap());
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                let active_clients: Vec<_> = {
                    clients_ref.read().values().map(|c| c.sender.clone()).collect()
                };
                for sender in active_clients {
                    let _ = sender.send(ping.clone()).await;
                }
            }
        });

        while let Ok((stream, addr)) = listener.accept().await {
            let server = self.clone();
            let orch = orchestrator.clone();
            let app = app_handle.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream, addr, orch, app).await {
                    log::warn!("Connection error with {}: {}", addr, e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
        addr: SocketAddr,
        orchestrator: Arc<CallOrchestrator>,
        app_handle: AppHandle,
    ) -> anyhow::Result<()> {
        let ws_stream = accept_async(stream).await?;
        log::info!("New WebSocket connection from: {}", addr);

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<Message>(32);

        // Task to forward messages to the socket
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Add client to map initially (without details)
        self.clients.write().insert(addr, ConnectedClient {
            id: addr,
            platform: "unknown".into(),
            name: "unknown".into(),
            sender: tx.clone(),
        });

        while let Some(msg) = read.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(_) => break, // Disconnected
            };

            if let Message::Text(text) = msg {
                if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                    self.handle_event(event, addr, &orchestrator, &app_handle, tx.clone()).await;
                }
            }
        }

        log::info!("WebSocket disconnected: {}", addr);
        let removed = self.clients.write().remove(&addr);
        if let Some(c) = removed {
            // Update frontend
            let _ = app_handle.emit("phone_disconnected", c.platform);
        }

        Ok(())
    }

    async fn handle_event(&self, event: WsEvent, addr: SocketAddr, orch: &CallOrchestrator, app: &AppHandle, tx: mpsc::Sender<Message>) {
        match event {
            WsEvent::Hello { client, version, device_name } => {
                log::info!("Client registered: {} ({}) v{}", device_name, client, version);
                if let Some(c) = self.clients.write().get_mut(&addr) {
                    c.platform = client.clone();
                    c.name = device_name.clone();
                }
                let _ = app.emit("phone_connected", serde_json::json!({
                    "platform": client,
                    "name": device_name,
                    "ip": addr.ip().to_string()
                }));
            }
            WsEvent::Heartbeat { .. } => {
                let _ = tx.send(Message::Text(serde_json::to_string(&WsEvent::HeartbeatAck).unwrap())).await;
            }
            WsEvent::HeartbeatAck => {}
            WsEvent::CallStarted { number, call_id, client } => {
                log::info!("Call started on {}: {} [{}]", client, number, call_id);
                orch.handle_call_started(number, call_id, client).await;
            }
            WsEvent::CallEnded { call_id, duration_sec, client } => {
                log::info!("Call ended on {}: {}s [{}]", client, duration_sec, call_id);
                orch.handle_call_ended(call_id).await;
            }
            WsEvent::BtConnected { device_name, client } => {
                log::info!("BT HFP connected on {}: {}", client, device_name);
                let _ = app.emit("bt_status", serde_json::json!({"platform": client, "connected": true}));
            }
            WsEvent::BtDisconnected { client } => {
                log::info!("BT HFP disconnected on {}", client);
                let _ = app.emit("bt_status", serde_json::json!({"platform": client, "connected": false}));
            }
            WsEvent::AgentModeChanged { enabled, client } => {
                log::info!("Agent mode {} on {}", if enabled {"ON"} else {"OFF"}, client);
                let _ = app.emit("agent_mode", serde_json::json!({"platform": client, "enabled": enabled}));
            }
            WsEvent::IncomingCall { number, client } => {
                log::info!("Incoming call on {}: {}", client, number);
                orch.handle_incoming_call(number, client).await;
            }
        }
    }

    pub async fn send_command(&self, platform: &str, cmd: WsCommand) -> anyhow::Result<()> {
        let msg = Message::Text(serde_json::to_string(&cmd)?);
        let mut sent = false;
        
        let target_senders: Vec<_> = {
            let clients = self.clients.read();
            clients.values()
                .filter(|c| c.platform == platform)
                .map(|c| c.sender.clone())
                .collect()
        };

        for sender in target_senders {
            let _ = sender.send(msg.clone()).await;
            sent = true;
        }
        
        if !sent {
            anyhow::bail!("No connected {} client found", platform);
        }
        Ok(())
    }
    
    pub fn get_connected_platforms(&self) -> Vec<String> {
        self.clients.read().values().map(|c| c.platform.clone()).collect()
    }
}
