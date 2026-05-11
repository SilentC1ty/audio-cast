# AudioCast 桌面端实现计划

## 模块架构

```
src-tauri/src/
├── main.rs              (已存在)
├── lib.rs               (扩展: run() 中注册所有 Tauri 命令)
├── protocol/
│   └── packet.rs        (UDP 包解析: SequenceID + Timestamp + PayloadLen + data)
├── network/
│   ├── mdns.rs          (mDNS 广播 _audiocast._udp.local)
│   ├── tcp.rs           (tokio TCP 握手服务器 :19090)
│   └── udp.rs           (UDP 接收 + 解析 + 送入 jitter buffer)
├── audio/
│   ├── jitter.rs        (动态抖动缓冲区, 60-120ms, 默认80ms)
│   ├── decoder.rs       (Opus 解码器, 48kHz stereo 10ms帧)
│   └── output.rs        (cpal 音频输出线程, WASAPI Exclusive 模式)
├── app/
│   └── state.rs         (AppState 共享状态, Tauri commands, 事件发送)
└── tauri_commands.rs    (所有 Tauri IPC 命令处理器)
```

## 线程模型

```
[mDNS 线程]         → 广播 _audiocast._udp.local
[TCP accept 线程]   → 接收握手 → 分配 UDP 端口 → 启动接收
[UDP recv 线程]     → 收包 → 解析头 → push 到 JitterBuffer
[Decoder 线程]      → 从 JitterBuffer pop → Opus decode → push 到 PCM Queue
[Audio 输出线程]    → 从 PCM Queue pop → 音量控制 → cpal 播放
[心跳线程]          → 每2秒计算延迟/丢包 → emit 事件到 Vue
```

线程间使用 `Arc<Mutex<VecDeque<T>>>` + `Arc<AtomicBool>` 通信和控制生命周期。

## 实现步骤

### 步骤 1: 添加依赖到 Cargo.toml

新增：
- `mdns-sd` — mDNS 服务广播
- `byteorder` — 网络字节序读写
- `uuid` — 会话 token
- `log` + `env_logger` — 日志

已有依赖不变：`tokio`, `opus`, `cpal`, `serde`, `serde_json`

### 步骤 2: 创建协议层 `protocol/packet.rs`

UDP 包格式（小端序）：
```
[0..4)   SequenceID  u32   包序号，用于丢包检测和重排
[4..12)  Timestamp   u64   采集时间戳，用于缓冲区对齐
[12..14) PayloadLen  u16   Opus 数据长度
[14..]   Opus Data   &[u8] Opus 编码音频数据
```

提供 `parse_packet(bytes: &[u8]) -> Option<AudioPacket>` 函数。

### 步骤 3: 创建抖动缓冲区 `audio/jitter.rs`

数据结构：
- `BinaryHeap<Reverse<JitterPacket>>` 按 timestamp 排序的最小堆
- 目标缓冲时长 60-120ms，默认 80ms
- 自动适配：网络稳定→缩小缓冲，网络抖动→增大缓冲

丢包策略：
- 单包丢失：返回空标记，让 Opus PLC 补偿
- 连续丢包 > 3 帧：填充静音
- 严重抖动：自动扩大缓冲窗口

### 步骤 4: 创建 Opus 解码器 `audio/decoder.rs`

- 48kHz, 2 channels, 10ms frame
- 解码单帧 → `PcmFrame { samples: Vec<i16>, timestamp: u64 }`
- 输出到 `Arc<Mutex<VecDeque<PcmFrame>>>` (PCM Queue)

### 步骤 5: 创建音频输出 `audio/output.rs`

- 使用 `cpal` 打开默认输出设备，优先 WASAPI Exclusive
- 独立输出线程：从 PCM Queue 取帧 → 音量系数 → 播放
- 音量范围 0.0 ~ 1.5（不影响系统主音量）
- 队列空时输出静音保护

### 步骤 6: 创建 UDP 接收器 `network/udp.rs`

- 绑定 UDP 端口
- 循环接收 → `protocol::packet::parse_packet()` 解析
- 更新统计信息 → `jitter.lock().push(packet)`

### 步骤 7: 创建 TCP 握手服务器 `network/tcp.rs`

- tokio async，监听 19090
- 握手请求：`{"action":"start", "sampleRate":48000, "channels":2}`
- 分配动态 UDP 端口 → 响应：`{"udpPort":19100, "bufferSize":80}`
- 仅接受 RFC1918 内网地址
- 生成 UUID session token

### 步骤 8: 创建 mDNS 广播 `network/mdns.rs`

- 服务类型：`_audiocast._udp.local`
- 广播内容：`{name, version:"1.0", tcp_port:19090}`
- 独立 daemon 线程

### 步骤 9: 创建 AppState `app/state.rs`

共享状态管理：
```rust
ConnectionState: Idle / Waiting / Connected / Streaming / Disconnected
NetworkStats: packets_received, packets_lost, latency_ms
AppState: connection, session_token, client_name, sample_rate, 
          channels, buffer_size_ms, volume(0.0-1.5), muted, stats
```

### 步骤 10: 创建 Tauri 命令 `tauri_commands.rs`

| 命令 | 方向 | 功能 |
|---|---|---|
| `start_server` | Vue→Rust | 启动 mDNS + TCP |
| `stop_server` | Vue→Rust | 停止所有服务 |
| `set_volume` | Vue→Rust | 音量 0.0-1.5 |
| `toggle_mute` | Vue→Rust | 静音切换 |
| `set_buffer_size` | Vue→Rust | 缓冲 ms |
| `get_status` | Vue→Rust | 获取完整状态 |
| `network-stats` | Rust→Vue | 实时网络统计 |
| `connection-changed` | Rust→Vue | 连接状态变更 |
| `device-info` | Rust→Vue | 设备信息 |

### 步骤 11: 更新 `lib.rs`

- 注册所有 Tauri 命令
- `setup()` 中可选自动启动 mDNS

### 步骤 12: 更新 Vue 前端

- **Pinia store** `src/stores/audio.ts` — 管理连接状态、音量、缓冲、网络统计
- **`StatusIndicator.vue`** — 连接状态圆点 + 设备名
- **`VolumeControl.vue`** — 音量滑块 + 静音按钮
- **`BufferControl.vue`** — 缓冲调节滑块
- **更新 `App.vue`** — 组合子组件，监听 Tauri 事件更新 UI

## 性能目标

| 指标 | 目标 |
|---|---|
| 端到端延迟 | 80-120ms |
| CPU 占用 | < 10% |
| 内存 | < 150MB |
| 丢包容忍 | 5% |
| 自动重连 | < 3s |
