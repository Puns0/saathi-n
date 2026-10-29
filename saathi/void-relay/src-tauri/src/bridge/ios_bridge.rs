use std::sync::Arc;
use crate::bridge::ws_server::{WsServer, WsCommand};
use anyhow::Result;

pub fn extract_ios_client(ws_server: &Arc<WsServer>) -> bool {
    let connected = ws_server.get_connected_platforms();
    connected.contains(&"ios".to_string())
}

pub async fn send_to_ios(ws_server: &Arc<WsServer>, command: WsCommand) -> Result<()> {
    ws_server.send_command("ios", command).await
}

pub fn get_ios_state(ws_server: &Arc<WsServer>) -> String {
    if extract_ios_client(ws_server) {
        "Connected".to_string()
    } else {
        "Disconnected".to_string()
    }
}
