use crate::audio::jitter::JitterBuffer;
use crate::protocol::packet;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// 启动 UDP 接收器
pub fn start_udp_receiver(
    port: u16,
    jitter: Arc<Mutex<JitterBuffer>>,
    running: Arc<AtomicBool>,
) -> Option<thread::JoinHandle<()>> {
    let addr = format!("0.0.0.0:{}", port);
    let socket = match UdpSocket::bind(&addr) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to bind UDP socket on {}: {}", addr, e);
            return None;
        }
    };

    if let Err(e) = socket.set_read_timeout(Some(std::time::Duration::from_millis(500))) {
        log::warn!("Failed to set UDP read timeout: {}", e);
    }

    log::info!("UDP receiver started on port {}", port);

    let handle = thread::Builder::new()
        .name("audiocast-udp".into())
        .spawn(move || {
            let mut buf = [0u8; 2048];

            while running.load(Ordering::Acquire) {
                match socket.recv_from(&mut buf) {
                    Ok((size, _src)) => {
                        if size < 14 {
                            log::warn!("UDP packet too short: {} bytes", size);
                            continue;
                        }

                        if let Some(audio_pkt) = packet::parse_packet(&buf[..size]) {
                            let mut jb = jitter.lock().unwrap();
                            jb.push(audio_pkt);

                            // 计算缓冲累计时间（每包 10ms）
                            jb.accumulate(10);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // 超时，正常循环检查
                    }
                    Err(e) => {
                        log::error!("UDP receive error: {}", e);
                    }
                }
            }

            log::info!("UDP receiver stopped on port {}", port);
        })
        .expect("Failed to spawn UDP receiver thread");

    Some(handle)
}
