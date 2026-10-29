use std::sync::Arc;
use crate::bridge::ws_server::{WsServer, WsCommand};
use anyhow::Result;

pub fn extract_android_client(ws_server: &Arc<WsServer>) -> bool {
    let connected = ws_server.get_connected_platforms();
    connected.contains(&"android".to_string())
}

pub async fn send_to_android(ws_server: &Arc<WsServer>, command: WsCommand) -> Result<()> {
    ws_server.send_command("android", command).await
}

pub fn get_android_state(ws_server: &Arc<WsServer>) -> String {
    if extract_android_client(ws_server) {
        "Connected".to_string()
    } else {
        "Disconnected".to_string()
    }
}
