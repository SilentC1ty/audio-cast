# AudioCast 移动端实现计划

## 当前状态 vs 设计要求

| 设计要求 | 当前状态 | 状态 |
|---|---|---|
| C++ Opus 编码 96kbps VBR 10ms 帧 | 原始 PCM 发送 | ❌ 缺失 |
| TCP 握手协议 | 无 | ❌ 缺失 |
| C++ RingBuffer + 锁无关队列 | 无 C++ 层 | ❌ 缺失 |
| Riverpod 状态管理 | StatefulWidget | ❌ 替换 |
| 实时延迟/丢包 | 硬编码 '--' | ❌ 缺失 |
| 3 线程模型 | 单线程 | ❌ 缺失 |
| mDNS 设备发现 | 已实现 | ✅ 可用 |
| MediaProjection + AudioPlaybackCapture | 已实现 | ✅ 可用 |
| 前台服务 + 通知 | 已实现 | ✅ 可用 |

## 目标架构

```
Flutter UI (Riverpod)
    ↓ MethodChannel
Kotlin Native Layer
    ├─ AudioCaptureService (前台服务)
    ├─ TcpHandshakeClient (TCP 握手 :19090)
    └─ AudioEngine (JNI 桥接)
          ↓ JNI
    C++ Audio Engine
        ├─ RingBuffer (环形缓冲)
        ├─ OpusEncoder (96kbps VBR)
        └─ UdpSender (带包头)
              ↓
        UDP → 桌面端
```

### 数据流路径
```
AudioRecord 回调 → PCM → RingBuffer → Opus Encode → UDP (加包头) → 桌面端
     (捕获线程)    (编码线程)    (发送线程)
```

## 实现步骤

### 步骤 1: 添加依赖
- **pubspec.yaml**: flutter_riverpod
- **build.gradle.kts**: `externalNativeBuild { cmake { ... } }`
- **CMakeLists.txt**: libopus + 自定义 C++ 库

### 步骤 2: Flutter 层重构
- `lib/models/device.dart` — 设备数据模型
- `lib/services/method_channel.dart` — MethodChannel 封装
- `lib/services/mdns_service.dart` — mDNS 发现封装
- `lib/stores/audio_store.dart` — Riverpod 状态管理
- `lib/pages/home_page.dart` — 重构主页 UI

### 步骤 3: TCP 握手客户端
- `TcpHandshakeClient.kt` — 连接 :19090，交换配置 JSON

### 步骤 4: C++ 音频引擎
- `ring_buffer.h/cpp` — PCM 环形缓冲区 (200ms)
- `opus_encoder.h/cpp` — Opus 编码器 (48kHz/2ch/96kbps/VBR)
- `udp_sender.h/cpp` — UDP 发送 (加包头: seq+ts+len)
- `jni_bridge.cpp` — JNI 入口函数

### 步骤 5: Kotlin JNI 桥接
- `AudioEngine.kt` — 封装 JNI native 方法

### 步骤 6: 重构 AudioCaptureService
- TCP 握手 → 获取 UDP 端口
- AudioRecord 回调 → AudioEngine.pushPCM()
- C++ 层自动编码+发送

### 步骤 7: 扩展 MethodChannel
- `getStatistics` → 延迟/丢包统计
- `startStreaming` → 先 TCP 握手再启动

### 步骤 8: 错误处理
- TCP 超时 → 回退 disconnected
- AudioRecord 中断 → 自动恢复
- Wi-Fi 切换 → 重新发现+重连

## 验证方式
1. `flutter pub get` — 依赖下载
2. `flutter build apk --debug` — 编译通过 (含 NDK/C++)
3. `flutter analyze` — 静态分析
