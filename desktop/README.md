# AudioCast Desktop

Tauri + Vue 3 桌面端，接收 Android 设备通过局域网发送的音频流并播放。

## 快速开始

```bash
pnpm install
pnpm tauri dev       # 开发模式
pnpm tauri build     # 打包
```

### 系统依赖

详情见[根目录文档](../README.md)。

## 架构

```
Vue UI ──IPC──→ Rust Backend
                   ├─ mDNS 广播 (服务发现)
                   ├─ TCP 握手 :19090
                   ├─ UDP 接收 :19100+
                   ├─ 抖动缓冲 (Jitter Buffer)
                   ├─ Opus 解码 (48kHz stereo)
                   └─ cpal 音频输出 (WASAPI/CoreAudio)
```

音频数据全程在 Rust 后端流转，不经过 Tauri IPC/WebView。

## Tauri IPC 命令

| 命令 | 方向 | 说明 |
|---|---|---|
| `start_server` | Vue→Rust | 启动 mDNS + TCP + 音频输出 |
| `stop_server` | Vue→Rust | 停止所有服务 |
| `set_volume` | Vue→Rust | 设置音量 0.0~1.5 |
| `toggle_mute` | Vue→Rust | 切换静音 |
| `set_buffer_size` | Vue→Rust | 设置缓冲区 60~120ms |
| `get_status` | Vue→Rust | 获取完整状态 |

`Rust→Vue` 事件: `network-stats`, `connection-changed`
