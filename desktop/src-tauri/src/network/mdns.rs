use mdns_sd::{DaemonEvent, ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

const SERVICE_TYPE: &str = "_audiocast._udp.local.";
const SERVICE_NAME: &str = "AudioCast Desktop";

/// 启动 mDNS 广播
pub fn start_mdns_broadcast(
    hostname: &str,
    tcp_port: u16,
    running: Arc<AtomicBool>,
) -> Option<thread::JoinHandle<()>> {
    let display_name = if hostname.is_empty() {
        SERVICE_NAME.to_string()
    } else {
        format!("{} ({})", SERVICE_NAME, hostname)
    };

    let daemon = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            log::error!("Failed to create mDNS daemon: {}", e);
            return None;
        }
    };

    let mut txt = HashMap::new();
    txt.insert("name".to_string(), display_name.clone());
    txt.insert("version".to_string(), "1.0".to_string());
    txt.insert("tcp_port".to_string(), tcp_port.to_string());

    let service_info = match ServiceInfo::new(
        SERVICE_TYPE,
        &display_name,
        &format!("{}.local.", hostname),
        "0.0.0.0",
        tcp_port,
        txt,
    ) {
        Ok(info) => info,
        Err(e) => {
            log::error!("Failed to create service info: {}", e);
            let _ = daemon.shutdown();
            return None;
        }
    };

    if let Err(e) = daemon.register(service_info) {
        log::error!("Failed to register mDNS service: {}", e);
        let _ = daemon.shutdown();
        return None;
    }

    log::info!(
        "mDNS broadcast started: {} type={} port={}",
        display_name,
        SERVICE_TYPE,
        tcp_port
    );

    let handle = thread::Builder::new()
        .name("audiocast-mdns".into())
        .spawn(move || {
            let receiver = match daemon.monitor() {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Failed to start mDNS monitor: {}", e);
                    let _ = daemon.shutdown();
                    return;
                }
            };

            loop {
                if !running.load(Ordering::Acquire) {
                    break;
                }
                match receiver.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(DaemonEvent::Error(e)) => {
                        log::error!("mDNS daemon error: {}", e);
                    }
                    _ => {}
                }
            }
            let _ = daemon.shutdown();
            log::info!("mDNS broadcast stopped");
        })
        .expect("Failed to spawn mDNS thread");

    Some(handle)
}
