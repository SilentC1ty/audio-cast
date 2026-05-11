use crate::app::state::{AppState, ConnectionState};
use crate::audio::jitter::JitterBuffer;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use uuid::Uuid;

const TCP_PORT: u16 = 19090;

/// 启动 TCP 握手服务器
pub async fn start_tcp_server(
    app: AppHandle,
    app_state: Arc<Mutex<AppState>>,
    running: Arc<AtomicBool>,
) {
    let addr = format!("0.0.0.0:{}", TCP_PORT);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind TCP server on {}: {}", addr, e);
            return;
        }
    };
    log::info!("TCP handshake server listening on {}", addr);

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, peer_addr) = match result {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("TCP accept error: {}", e);
                        continue;
                    }
                };
                let state = app_state.clone();
                let em = app.clone();
                tokio::spawn(async move {
                    handle_connection(stream, peer_addr, state, em).await;
                });
            }
            _ = async {
                while running.load(std::sync::atomic::Ordering::Acquire) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            } => {
                break;
            }
        }
    }
    log::info!("TCP server stopped");
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    app_state: Arc<Mutex<AppState>>,
    app: AppHandle,
) {
    if !is_private_ip(&peer_addr) {
        log::warn!("Rejected connection from non-private IP: {}", peer_addr);
        return;
    }

    log::info!("TCP connection from: {}", peer_addr);

    // 分配 UDP 端口
    let udp_port = {
        let mut state = app_state.lock().unwrap();
        let port = state.udp_port + 1;
        state.udp_port = port;
        port
    };

    let token = Uuid::new_v4().to_string();

    let resp = serde_json::json!({
        "udpPort": udp_port,
        "bufferSize": 80,
        "token": token,
    });

    if let Err(e) = stream
        .write_all(resp.to_string().as_bytes())
        .await
    {
        log::error!("Failed to send handshake response: {}", e);
        return;
    }

    {
        let mut state = app_state.lock().unwrap();
        state.connection = ConnectionState::Connected;
        state.client_ip = Some(peer_addr.ip().to_string());
        state.session_token = Some(token);
    }

    let _ = app.emit(
        "connection-changed",
        serde_json::json!({"state": "connected", "client": peer_addr.ip().to_string()}),
    );

    log::info!("Handshake complete: client={}, udp_port={}", peer_addr.ip(), udp_port);
}

fn is_private_ip(addr: &SocketAddr) -> bool {
    match addr.ip() {
        std::net::IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 10
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 192 && o[1] == 168)
                || o[0] == 127
        }
        std::net::IpAddr::V6(_) => true,
    }
}
