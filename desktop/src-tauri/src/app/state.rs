use serde::Serialize;

/// 连接状态
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ConnectionState {
    Idle,
    Waiting,
    Connected,
    Streaming,
    Disconnected,
}

impl ConnectionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionState::Idle => "idle",
            ConnectionState::Waiting => "waiting",
            ConnectionState::Connected => "connected",
            ConnectionState::Streaming => "streaming",
            ConnectionState::Disconnected => "disconnected",
        }
    }
}

/// 网络统计
#[derive(Debug, Clone, Serialize)]
pub struct NetworkStats {
    pub packets_received: u64,
    pub packets_lost: u64,
    pub latency_ms: u32,
    pub packet_loss_pct: f32,
}

/// 应用状态
pub struct AppState {
    pub connection: ConnectionState,
    pub session_token: Option<String>,
    pub client_ip: Option<String>,
    pub client_name: Option<String>,
    pub sample_rate: u32,
    pub channels: u8,
    pub buffer_size_ms: u32,
    pub volume: f32,
    pub muted: bool,
    pub stats: NetworkStats,
    pub udp_port: u16,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            connection: ConnectionState::Idle,
            session_token: None,
            client_ip: None,
            client_name: None,
            sample_rate: 48000,
            channels: 2,
            buffer_size_ms: 80,
            volume: 1.0,
            muted: false,
            stats: NetworkStats {
                packets_received: 0,
                packets_lost: 0,
                latency_ms: 0,
                packet_loss_pct: 0.0,
            },
            udp_port: 19099,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "connection": self.connection.as_str(),
            "latency": self.stats.latency_ms,
            "packetLoss": self.stats.packet_loss_pct,
            "bufferSize": self.buffer_size_ms,
            "volume": self.volume,
            "muted": self.muted,
            "clientName": self.client_name,
        })
    }
}
