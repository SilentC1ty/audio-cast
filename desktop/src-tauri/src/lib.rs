use std::sync::Mutex;
use tauri::State;

struct AudioState {
    buffer_size_ms: u32,
}

fn run_audio_loop() {
    // TODO: 实现 UDP 接收 -> Opus 解码 -> cpal 播放的音频管线
}

#[tauri::command]
fn get_status() -> String {
    serde_json::json!({"status": "idle", "buffer_ms": 60}).to_string()
}

#[tauri::command]
fn set_buffer_size(buffer_ms: u32, state: State<Mutex<AudioState>>) -> String {
    if let Ok(mut s) = state.lock() {
        s.buffer_size_ms = buffer_ms;
    }
    serde_json::json!({"status": "ok", "buffer_ms": buffer_ms}).to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AudioState { buffer_size_ms: 60 }))
        .invoke_handler(tauri::generate_handler![get_status, set_buffer_size])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
