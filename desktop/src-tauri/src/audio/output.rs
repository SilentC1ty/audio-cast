use crate::app::state::AppState;
use crate::audio::decoder::{PcmFrame, SAMPLE_RATE, CHANNELS, FRAME_DURATION_MS};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// 启动音频输出线程
///
/// 从 PCM 队列拉取数据，经音量控制后送至 cpal 播放。
/// 音量/静音状态从 AppState 读取。
pub fn start_output_thread(
    pcm_queue: Arc<Mutex<VecDeque<PcmFrame>>>,
    app_state: Arc<Mutex<AppState>>,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("audiocast-output".into())
        .spawn(move || {
            let host = cpal::default_host();

            let device = match host.default_output_device() {
                Some(d) => d,
                None => {
                    log::error!("No default audio output device found");
                    return;
                }
            };

            log::info!("Audio output device: {}", device.name().unwrap_or_default());

            let config = StreamConfig {
                channels: CHANNELS,
                sample_rate: cpal::SampleRate(SAMPLE_RATE),
                buffer_size: cpal::BufferSize::Default,
            };

            let err_fn = move |err| {
                log::error!("Audio output error: {}", err);
            };

            let stream = match device.build_output_stream(
                &config,
                move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    let pcm = app_state.lock().unwrap();
                    let vol = pcm.volume;
                    let is_muted = pcm.muted;
                    drop(pcm);

                    let mut queue = pcm_queue.lock().unwrap();

                    let mut out_idx = 0;
                    while out_idx < data.len() {
                        if let Some(frame) = queue.front() {
                            let available = frame.samples.len();
                            let needed = data.len() - out_idx;
                            let to_copy = available.min(needed);

                            if is_muted || vol <= 0.0 {
                                data[out_idx..out_idx + to_copy].fill(0.0);
                            } else {
                                for i in 0..to_copy {
                                    data[out_idx + i] =
                                        frame.samples[i] as f32 / 32768.0 * vol;
                                }
                            }
                            out_idx += to_copy;
                            queue.pop_front();
                        } else {
                            // 无数据：填充静音
                            let fill = data.len() - out_idx;
                            data[out_idx..out_idx + fill].fill(0.0);
                            out_idx = data.len();
                        }
                    }
                },
                err_fn,
                None,
            ) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to build output stream: {}", e);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                log::error!("Failed to play audio stream: {}", e);
                return;
            }

            log::info!("Audio output started");

            while running.load(Ordering::Acquire) {
                thread::sleep(std::time::Duration::from_millis(100));
            }

            drop(stream);
            log::info!("Audio output stopped");
        })
        .expect("Failed to spawn audio output thread")
}
