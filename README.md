# AudioCast

局域网音频转发工具 — 将 Android 设备音频无线串流至桌面端播放。

## 项目结构

```
audio-cast/
├── mobile/          # Flutter 移动端 (Android 音频发送端)
│   ├── lib/         # Dart UI (设备发现 + 状态显示)
│   └── android/     # Kotlin 原生层 (MediaProjection 捕获 + 前台服务)
│
├── desktop/         # Tauri + Vue 桌面端 (音频接收端)
│   ├── src/         # Vue 3 + TailwindCSS 前端
│   └── src-tauri/   # Rust 后端 (UDP 接收 + Opus 解码 + cpal 播放)
│
└── AudioCast_PRD_Flutter_Tauri_RequirementsFocus.md  # 需求文档
```

## 桌面端 (Tauri + Vue)

### 系统要求

- Rust 1.70+
- Node.js 18+
- pnpm 9+

#### Linux 额外依赖

```bash
sudo apt-get install -y \
  build-essential \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  librsvg2-dev \
  patchelf \
  libasound2-dev
```

#### Windows

- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (Win10+ 自带)
- [Microsoft Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 或 Visual Studio (勾选"使用 C++ 的桌面开发")

#### macOS

- Xcode Command Line Tools: `xcode-select --install`

### 编译 & 运行

```bash
cd desktop
pnpm install
pnpm tauri dev       # 开发模式
```

### 打包

```bash
pnpm tauri build     # 生成安装包 (Windows: .msi, macOS: .dmg, Linux: .deb/.AppImage)
```

打包产物位于 `desktop/src-tauri/target/release/bundle/`。

## 移动端 (Flutter)

### 系统要求

- Flutter 3.29+
- Android Studio (Android SDK API 29+)
- Android 10+ 设备或模拟器

### 编译 & 运行

```bash
cd mobile
flutter pub get
flutter run           # 安装到已连接的 Android 设备
```

### 打包 APK

```bash
flutter build apk --release
# 或拆分 AAB
flutter build appbundle --release
```

## 使用流程

1. 桌面端启动 AudioCast，点击"启动服务"
2. 手机端打开 AudioCast App，自动扫描发现桌面端
3. 点击桌面端设备名称，授权 MediaProjection 录屏权限
4. 授权后自动开始音频流转

## 技术栈

| 端 | 技术 | 职责 |
|---|---|---|
| 移动端 UI | Flutter (Dart) | 设备发现、状态管理、连接控制 |
| 原生捕获 | Kotlin + C++ | MediaProjection API、Opus 编码、UDP 发送 |
| 桌面端 UI | Vue 3 + TailwindCSS | 音量/缓冲控制、实时状态显示 |
| 桌面端引擎 | Rust (Tauri) | UDP 接收、Opus 解码、cpal 音频输出 |

### 通信协议

| 类型 | 协议 | 用途 |
|---|---|---|
| 设备发现 | mDNS | `_audiocast._udp.local` 服务广播 |
| 握手 | TCP :19090 | 协商配置、分配 UDP 端口 |
| 音频流 | UDP :19100+ | Opus 编码音频帧 (10ms/帧) |

## 性能目标

| 指标 | 目标 |
|---|---|
| 端到端延迟 | 80ms ~ 120ms |
| 桌面端 CPU | < 10% |
| 内存占用 | < 150MB |
| 丢包容忍 | 5% |
| 自动重连 | < 3s |
