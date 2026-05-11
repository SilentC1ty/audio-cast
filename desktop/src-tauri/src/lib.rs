mod app;
mod audio;
mod network;
mod protocol;

use crate::app::state::AppState;
use crate::audio::decoder::PcmFrame;
use crate::audio::jitter::JitterBuffer;
use crate::audio::output;
use crate::network::{mdns, tcp};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

// ─── Tauri Commands ────────────────────────────────────────────────────────────

/// 启动服务：mDNS 广播 + TCP 握手 + 音频输出
#[tauri::command]
async fn start_server(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    jitter: State<'_, Arc<Mutex<JitterBuffer>>>,
    running: State<'_, Arc<AtomicBool>>,
    pcm_queue: State<'_, Arc<Mutex<VecDeque<PcmFrame>>>>,
) -> Result<String, String> {
    if running.load(Ordering::Acquire) {
        return Err("Server already running".into());
    }

    running.store(true, Ordering::Release);

    let r = Arc::clone(running.inner());
    let jb = Arc::clone(jitter.inner());
    let st = Arc::clone(state.inner());
    let pq = Arc::clone(pcm_queue.inner());

    // mDNS 广播
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "desktop".into());
    let _mdns_handle = mdns::start_mdns_broadcast(&hostname, 19090, r.clone());

    // 音频输出线程
    let _output_handle = output::start_output_thread(pq, st.clone(), r.clone());

    // TCP 握手服务器（tokio async）
    let tcp_state = st.clone();
    let tcp_running = r.clone();
    let tcp_app = app.clone();
    tokio::spawn(async move {
        tcp::start_tcp_server(tcp_app, tcp_state, tcp_running).await;
    });

    // 心跳线程
    let hb_jitter = jb.clone();
    let hb_state = st.clone();
    let hb_app = app.clone();
    let hb_running = r.clone();
    thread::Builder::new()
        .name("audiocast-heartbeat".into())
        .spawn(move || {
            while hb_running.load(Ordering::Acquire) {
                thread::sleep(std::time::Duration::from_secs(2));

                let (latency, packet_loss) = {
                    let jl = hb_jitter.lock().unwrap();
                    let rate = jl.packet_loss_rate();
                    if jl.packets_received() + jl.packets_lost() > 0 {
                        (jl.current_ms(), rate * 100.0)
                    } else {
                        (0u32, 0.0)
                    }
                };

                {
                    let mut s = hb_state.lock().unwrap();
                    s.stats.latency_ms = latency;
                    s.stats.packet_loss_pct = packet_loss;
                    let jl = hb_jitter.lock().unwrap();
                    s.stats.packets_received = jl.packets_received();
                    s.stats.packets_lost = jl.packets_lost();
                }

                let _ = hb_app.emit(
                    "network-stats",
                    serde_json::json!({
                        "latency": latency,
                        "packetLoss": packet_loss,
                    }),
                );
            }
        })
        .expect("Failed to spawn heartbeat thread");

    let _ = app.emit(
        "connection-changed",
        serde_json::json!({"state": "waiting"}),
    );

    Ok(serde_json::json!({"status": "started"}).to_string())
}

/// 停止所有服务
#[tauri::command]
async fn stop_server(
    running: State<'_, Arc<AtomicBool>>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    running.store(false, Ordering::Release);
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.connection = crate::app::state::ConnectionState::Idle;
    }
    Ok(serde_json::json!({"status": "stopped"}).to_string())
}

/// 设置音量 (0.0 ~ 1.5)
#[tauri::command]
async fn set_volume(
    value: f64,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let vol = value.clamp(0.0, 1.5) as f32;
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.volume = vol;
    Ok(serde_json::json!({"status": "ok", "volume": vol}).to_string())
}

/// 切换静音
#[tauri::command]
async fn toggle_mute(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.muted = !s.muted;
    Ok(serde_json::json!({"status": "ok", "muted": s.muted}).to_string())
}

/// 设置缓冲区大小 (ms)
#[tauri::command]
async fn set_buffer_size(
    ms: u32,
    state: State<'_, Arc<Mutex<AppState>>>,
    jitter: State<'_, Arc<Mutex<JitterBuffer>>>,
) -> Result<String, String> {
    let ms_clamped = ms.clamp(60, 120);
    {
        let mut jb = jitter.lock().map_err(|e| e.to_string())?;
        jb.set_target_ms(ms_clamped);
    }
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.buffer_size_ms = ms_clamped;
    }
    Ok(serde_json::json!({"status": "ok", "bufferSize": ms_clamped}).to_string())
}

/// 获取完整状态
#[tauri::command]
async fn get_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.to_json().to_string())
}

// ─── Entry Point ────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(Mutex::new(AppState::new())))
        .manage(Arc::new(Mutex::new(JitterBuffer::new(80))))
        .manage(Arc::new(AtomicBool::new(false)))
        .manage(Arc::new(Mutex::new(VecDeque::<PcmFrame>::new())))
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            set_volume,
            toggle_mute,
            set_buffer_size,
            get_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
