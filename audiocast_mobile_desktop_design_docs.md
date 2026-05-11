# AudioCast 详细设计文档（移动端 / 桌面端）

基于 PRD《AudioCast 产品需求文档》编写。

---

# 一、项目总体设计

## 1.1 项目目标

AudioCast 是一款局域网音频无线转发工具。

系统由两部分组成：

- 移动端（Android + Flutter）
  - 负责捕获 Android 系统音频
  - 编码并通过 UDP 推送到桌面端
- 桌面端（Tauri + Rust + Vue）
  - 负责接收音频流
  - 解码并播放
  - 提供状态监控与控制能力

核心目标：

- 局域网自动发现
- 一键连接
- 全局音频捕获
- 80ms~120ms 低延迟
- 后台稳定运行
- 高稳定性与低资源占用

---

## 1.2 总体架构

```text
┌─────────────────────────────────────┐
│            Android Phone            │
├─────────────────────────────────────┤
│ Flutter UI                          │
│  ├─ 设备发现                        │
│  ├─ 状态显示                        │
│  └─ 连接控制                        │
├─────────────────────────────────────┤
│ Kotlin Native Layer                 │
│  ├─ MediaProjection                 │
│  ├─ AudioPlaybackCapture            │
│  ├─ ForegroundService               │
│  └─ UDP Socket                      │
├─────────────────────────────────────┤
│ C++ Audio Engine                    │
│  ├─ PCM RingBuffer                  │
│  ├─ Opus Encoder                    │
│  └─ Packet Builder                  │
└─────────────────────────────────────┘
                  │
                  │ UDP Opus Stream
                  ▼
┌─────────────────────────────────────┐
│              Desktop                │
├─────────────────────────────────────┤
│ Vue3 UI                             │
│  ├─ 音量控制                         │
│  ├─ Buffer 调节                     │
│  ├─ 网络状态                         │
│  └─ 设备连接管理                     │
├─────────────────────────────────────┤
│ Tauri Rust Backend                  │
│  ├─ mDNS Service                    │
│  ├─ TCP Handshake                   │
│  ├─ UDP Receiver                    │
│  ├─ Jitter Buffer                   │
│  ├─ Opus Decoder                    │
│  └─ Audio Output                    │
└─────────────────────────────────────┘
```

---

# 二、移动端详细设计（Flutter + Android Native）

# 2.1 技术架构

## 2.1.1 分层设计

```text
Flutter UI Layer
    ↓
Flutter Service Layer
    ↓ MethodChannel / FFI
Android Native Layer (Kotlin)
    ↓ JNI
C++ Audio Engine
```

各层职责：

| 层级 | 职责 |
|---|---|
| Flutter UI | 页面展示、状态管理、设备列表 |
| Flutter Service | 与 Native 通信 |
| Kotlin Layer | 权限管理、音频捕获、后台服务 |
| C++ Layer | Opus 编码、实时发送 |

---

# 2.2 Flutter 模块设计

## 2.2.1 页面结构

### 首页 HomePage

功能：

- 显示当前连接状态
- 显示网络状态
- 显示延迟与丢包率
- 展示设备列表
- 控制开始/停止推流

页面状态：

```text
未连接
搜索设备中
连接中
已连接
推流中
异常断开
```

---

## 2.2.2 状态管理

推荐：Riverpod

状态对象：

```dart
class AppState {
  bool connected;
  bool streaming;
  String deviceName;
  int latency;
  double packetLoss;
}
```

---

## 2.2.3 Native 通信接口

Flutter 仅用于控制，不传输 PCM 数据。

MethodChannel 接口：

| 方法 | 说明 |
|---|---|
| startCapture | 开始录制 |
| stopCapture | 停止录制 |
| scanDevices | 扫描桌面端 |
| connectDevice | 建立连接 |
| getStatistics | 获取网络统计 |

示例：

```dart
await methodChannel.invokeMethod('startCapture');
```

---

# 2.3 Android Native 设计

# 2.3.1 权限体系

所需权限：

```xml
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
```

ForegroundService：

```xml
android:foregroundServiceType="mediaProjection"
```

---

# 2.3.2 MediaProjection 流程

流程：

```text
用户点击开始
    ↓
请求 MediaProjection 授权
    ↓
系统弹窗授权
    ↓
创建 AudioPlaybackCapture
    ↓
创建 AudioRecord
    ↓
启动 ForegroundService
    ↓
启动采集线程
```

---

# 2.3.3 AudioRecord 配置

推荐参数：

| 参数 | 值 |
|---|---|
| SampleRate | 48000 |
| Channel | Stereo |
| Encoding | PCM 16bit |
| BufferSize | 2~3 倍 frame size |

示例：

```kotlin
AudioFormat.Builder()
    .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
    .setSampleRate(48000)
    .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
```

---

# 2.3.4 Foreground Service

职责：

- 保活
- 防止 Doze 杀进程
- 持续音频采集
- 网络发送

通知栏内容：

```text
AudioCast 正在转发系统音频
目标设备：DESKTOP-01
当前延迟：92ms
```

---

# 2.4 C++ 音频引擎设计

# 2.4.1 设计目标

避免：

```text
AudioRecord -> MethodChannel -> Dart -> Socket
```

正确链路：

```text
AudioRecord
    ↓
JNI
    ↓
C++ RingBuffer
    ↓
Opus Encoder
    ↓
UDP Socket
```

---

# 2.4.2 PCM RingBuffer

目的：

- 解耦录制线程与编码线程
- 避免阻塞
- 平滑突发数据

结构：

```cpp
struct PCMFrame {
    uint64_t timestamp;
    int16_t samples[FRAME_SIZE];
};
```

RingBuffer 推荐长度：

```text
100ms ~ 200ms
```

---

# 2.4.3 Opus 编码配置

推荐参数：

| 参数 | 值 |
|---|---|
| Sample Rate | 48000 |
| Channels | 2 |
| Frame Duration | 10ms |
| Bitrate | 96kbps |
| Complexity | 5 |
| VBR | 开启 |

原因：

- 10ms 帧长延迟更低
- 96kbps 足够高质量
- 中等复杂度减少 CPU 占用

---

# 2.4.4 UDP 封包结构

```text
┌────────────┬────────────┬────────────┬────────────┐
│ SequenceID │ Timestamp  │ PayloadLen │ Opus Data  │
└────────────┴────────────┴────────────┴────────────┘
```

字段：

| 字段 | 类型 | 说明 |
|---|---|---|
| SequenceID | uint32 | 包序号 |
| Timestamp | uint64 | 采集时间 |
| PayloadLen | uint16 | 数据长度 |
| OpusData | byte[] | Opus 数据 |

---

# 2.4.5 网络线程模型

推荐三线程：

```text
采集线程
编码线程
发送线程
```

职责：

| 线程 | 功能 |
|---|---|
| Capture | AudioRecord 读取 PCM |
| Encode | Opus 编码 |
| Send | UDP 发送 |

线程间使用 lock-free queue。

---

# 2.5 mDNS 自动发现

## 2.5.1 发现流程

```text
桌面端广播 _audiocast._udp.local
    ↓
手机端扫描局域网服务
    ↓
解析 IP 与端口
    ↓
展示设备列表
```

服务字段：

| Key | Value |
|---|---|
| service | _audiocast._udp.local |
| name | DESKTOP-01 |
| version | 1.0 |
| tcp_port | 19090 |

---

# 2.6 TCP 握手设计

## 2.6.1 握手流程

```text
Flutter -> Desktop
{
  action: start,
  sampleRate: 48000,
  channels: 2
}

Desktop -> Flutter
{
  udpPort: 19100,
  bufferSize: 80
}
```

目的：

- 分配 UDP 端口
- 同步配置
- 协商 buffer 大小

---

# 2.7 异常处理

## 2.7.1 网络异常

处理策略：

| 场景 | 处理 |
|---|---|
| UDP 超时 | 自动重连 |
| Wi-Fi 切换 | 自动重新发现 |
| IP 变化 | 重新 TCP 握手 |

---

## 2.7.2 音频异常

| 场景 | 处理 |
|---|---|
| AudioRecord 中断 | 自动恢复 |
| 权限被回收 | 提示重新授权 |
| MediaProjection 失效 | 停止服务 |

---

# 2.8 性能优化

## 2.8.1 CPU 优化

原则：

- 不进入 Dart 高频数据流
- 使用 native buffer
- 避免频繁内存复制
- 使用对象池

---

## 2.8.2 延迟优化

关键点：

| 模块 | 延迟控制 |
|---|---|
| AudioRecord | 小 buffer |
| Opus | 10ms frame |
| UDP | 无重传 |
| JitterBuffer | 60~80ms |
| AudioOutput | exclusive 模式 |

---

# 2.9 日志设计

日志等级：

```text
DEBUG
INFO
WARN
ERROR
```

日志分类：

| 分类 | 内容 |
|---|---|
| audio | 音频采集 |
| network | UDP/TCP |
| mdns | 服务发现 |
| codec | Opus |
| system | 权限/生命周期 |

---

# 三、桌面端详细设计（Tauri + Rust + Vue）

# 3.1 技术架构

```text
Vue UI
   ↓
Tauri IPC
   ↓
Rust Backend
   ├─ mDNS
   ├─ TCP Server
   ├─ UDP Receiver
   ├─ Jitter Buffer
   ├─ Opus Decoder
   └─ CPAL Audio Output
```

---

# 3.2 Vue UI 设计

# 3.2.1 主窗口布局

布局：

```text
┌──────────────────────┐
│ AudioCast            │
├──────────────────────┤
│ 状态：已连接          │
│ 手机：Pixel 9         │
│ 延迟：93ms            │
│ 丢包：0.4%            │
├──────────────────────┤
│ 音量滑块              │
│ Buffer 滑块           │
│ 静音按钮              │
├──────────────────────┤
│ 日志/高级设置         │
└──────────────────────┘
```

---

# 3.2.2 状态管理

推荐：Pinia

Store：

```typescript
interface AudioState {
  connected: boolean
  latency: number
  packetLoss: number
  bufferSize: number
  volume: number
}
```

---

# 3.2.3 Tauri IPC

调用：

```typescript
invoke('set_volume', { value: 0.8 })
```

事件监听：

```typescript
listen('network-stats', callback)
```

---

# 3.3 Rust Backend 设计

# 3.3.1 模块划分

```text
src-tauri/
├─ network/
│   ├─ mdns.rs
│   ├─ tcp.rs
│   └─ udp.rs
├─ audio/
│   ├─ jitter.rs
│   ├─ decoder.rs
│   └─ output.rs
├─ protocol/
│   └─ packet.rs
└─ app/
    └─ state.rs
```

---

# 3.3.2 mDNS 广播

广播服务：

```text
_audiocast._udp.local
```

广播内容：

```json
{
  "name": "DESKTOP-01",
  "version": "1.0",
  "tcp_port": 19090
}
```

---

# 3.3.3 TCP Server

职责：

- 接收握手
- 分配 UDP 端口
- 保存会话状态

推荐：tokio async。

---

# 3.3.4 UDP Receiver

流程：

```text
UDP Socket
    ↓
Packet Parser
    ↓
Sequence Check
    ↓
Jitter Buffer
```

校验：

| 校验 | 目的 |
|---|---|
| Sequence | 丢包检测 |
| Timestamp | 排序 |
| PayloadSize | 数据合法性 |

---

# 3.4 Jitter Buffer 设计

# 3.4.1 核心目标

解决：

- Wi-Fi 抖动
- UDP 乱序
- 短时丢包

---

# 3.4.2 Buffer 结构

```rust
struct JitterPacket {
    sequence: u32,
    timestamp: u64,
    payload: Vec<u8>,
}
```

队列：

```text
最小堆 / Ring Queue
```

---

# 3.4.3 动态 Buffer 算法

逻辑：

```text
网络稳定 → 缩小 buffer
网络抖动 → 增大 buffer
```

范围：

```text
60ms ~ 120ms
```

默认：

```text
80ms
```

---

# 3.4.4 丢包处理

策略：

| 丢包情况 | 处理 |
|---|---|
| 单包丢失 | PLC 补偿 |
| 连续丢失 | 静音填充 |
| 严重抖动 | 扩大 Buffer |

Opus 自带 PLC（Packet Loss Concealment）。

---

# 3.5 Opus 解码

# 3.5.1 解码配置

| 参数 | 值 |
|---|---|
| Sample Rate | 48000 |
| Channels | 2 |
| Frame Size | 10ms |

---

# 3.5.2 解码线程

流程：

```text
JitterBuffer
    ↓
Opus Decoder
    ↓
PCM Queue
    ↓
Audio Output
```

---

# 3.6 音频输出设计

# 3.6.1 CPAL 输出

推荐：

```text
WASAPI Exclusive
```

原因：

- 更低延迟
- 更稳定
- 更少系统混音影响

macOS：

```text
CoreAudio
```

---

# 3.6.2 输出线程

职责：

- 从 PCM Queue 拉取数据
- 音量控制
- 静音控制
- 推送给系统设备

---

# 3.6.3 音量控制

实现：

```text
PCM Sample × VolumeFactor
```

范围：

```text
0.0 ~ 1.5
```

不修改系统音量。

---

# 3.7 会话管理

# 3.7.1 Session 对象

```rust
struct Session {
    client_ip: String,
    connected: bool,
    sample_rate: u32,
    channels: u8,
    latency: u32,
}
```

---

# 3.7.2 心跳机制

每 2 秒：

```text
PING
```

响应：

```text
PONG
```

用途：

- 计算 RTT
- 判断断线
- 更新 UI

---

# 3.8 性能优化

# 3.8.1 Rust 优化

原则：

- 避免频繁 Vec 分配
- 使用 bytes crate
- 减少 clone
- lock-free queue
- tokio async

---

# 3.8.2 音频优化

重点：

| 项目 | 优化 |
|---|---|
| Decoder | 复用实例 |
| Buffer | 预分配 |
| Output | 独立线程 |
| Queue | 无锁队列 |

---

# 3.9 安全设计

# 3.9.1 局域网限制

仅允许：

```text
RFC1918 内网地址
```

拒绝公网连接。

---

# 3.9.2 握手校验

握手 Token：

```text
UUID Session Token
```

避免其他设备误连接。

---

# 3.10 日志系统

日志文件：

```text
logs/app.log
```

日志分类：

| 模块 | 内容 |
|---|---|
| network | TCP/UDP |
| jitter | 缓冲区 |
| decoder | Opus 解码 |
| output | 音频输出 |
| session | 会话 |

---

# 四、关键时序流程

# 4.1 设备发现流程

```text
Desktop 启动
    ↓
mDNS 广播
    ↓
Mobile 扫描
    ↓
发现 Desktop
    ↓
展示设备列表
```

---

# 4.2 音频推流流程

```text
用户点击连接
    ↓
TCP 握手
    ↓
Desktop 返回 UDP Port
    ↓
启动 AudioRecord
    ↓
PCM 捕获
    ↓
Opus 编码
    ↓
UDP 发送
    ↓
Desktop 接收
    ↓
Jitter Buffer
    ↓
Opus 解码
    ↓
CPAL 输出
```

---

# 4.3 异常恢复流程

```text
Wi-Fi 断开
    ↓
UDP Timeout
    ↓
自动停止输出
    ↓
重新扫描 mDNS
    ↓
重新 TCP 握手
    ↓
恢复推流
```

---

# 五、性能指标

| 指标 | 目标 |
|---|---|
| 端到端延迟 | 80ms~120ms |
| CPU 占用（手机） | < 15% |
| CPU 占用（桌面） | < 10% |
| 内存占用 | < 150MB |
| 丢包容忍 | 5% |
| 自动重连时间 | < 3s |

---

# 六、测试方案

# 6.1 功能测试

| 测试项 | 内容 |
|---|---|
| 音频捕获 | 系统声音是否正常 |
| 自动发现 | 是否自动显示设备 |
| 自动重连 | Wi-Fi 切换恢复 |
| 后台保活 | 熄屏后持续播放 |

---

# 6.2 性能测试

| 测试项 | 内容 |
|---|---|
| 延迟 | 视频口型同步 |
| 抖动 | 弱 Wi-Fi 场景 |
| CPU | 长时间运行 |
| 内存泄漏 | 24 小时运行 |

---

# 6.3 兼容性测试

Android：

- Android 10
- Android 11
- Android 12
- Android 13
- Android 14

Windows：

- Windows 10
- Windows 11

macOS：

- Intel
- Apple Silicon

---

# 七、后续扩展方向

## 7.1 功能扩展

未来可增加：

- 麦克风回传
- 双向语音
- 蓝牙耳机自动切换
- 多设备同步播放
- WebRTC 模式
- AES 音频加密
- 跨公网连接

---

## 7.2 技术演进

后续可考虑：

- QUIC 替代 UDP + TCP
- Rust 全平台统一音频引擎
- FFmpeg 音频处理链
- WebAssembly 控制台

---

# 八、结论

AudioCast 的核心挑战并不在 UI，而在：

- Android 系统级音频捕获
- 低延迟实时音频链路
- UDP 抖动控制
- Native 音频线程调度
- 后台稳定运行

本方案通过：

- Flutter + Native 解耦
- C++ Opus 实时编码
- Rust 高性能接收
- 动态 Jitter Buffer
- CPAL 低延迟输出

能够较好满足：

```text
80ms~120ms
稳定低延迟局域网音频转发
```

这一核心目标。

