import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type ConnectionState = "idle" | "waiting" | "connected" | "streaming" | "disconnected";

export const useAudioStore = defineStore("audio", () => {
  const connection = ref<ConnectionState>("idle");
  const latency = ref(0);
  const packetLoss = ref(0);
  const bufferSize = ref(80);
  const volume = ref(1.0);
  const muted = ref(false);
  const deviceName = ref("");

  const statusText = computed(() => {
    const map: Record<ConnectionState, string> = {
      idle: "未连接",
      waiting: "等待连接...",
      connected: "已连接",
      streaming: "音频流转中",
      disconnected: "连接断开",
    };
    return map[connection.value] || connection.value;
  });

  const statusColor = computed(() => {
    const map: Record<ConnectionState, string> = {
      idle: "bg-gray-500",
      waiting: "bg-yellow-500",
      connected: "bg-green-500",
      streaming: "bg-green-500",
      disconnected: "bg-red-500",
    };
    return map[connection.value] || "bg-gray-500";
  });

  let unlisteners: (() => void)[] = [];

  async function init() {
    unlisteners.push(
      await listen<{ state: ConnectionState; client?: string }>("connection-changed", (e) => {
        connection.value = e.payload.state;
        if (e.payload.client) deviceName.value = e.payload.client;
      }),
    );
    unlisteners.push(
      await listen<{ latency: number; packetLoss: number }>("network-stats", (e) => {
        latency.value = e.payload.latency;
        packetLoss.value = e.payload.packetLoss;
      }),
    );
    // 初始化状态
    try {
      const res = await invoke<string>("get_status");
      const data = JSON.parse(res);
      connection.value = data.connection || "idle";
      bufferSize.value = data.bufferSize || 80;
      volume.value = data.volume ?? 1.0;
      muted.value = data.muted ?? false;
    } catch {
      // 服务未启动，默认 idle
    }
  }

  function cleanup() {
    unlisteners.forEach((fn) => fn());
    unlisteners = [];
  }

  async function startServer() {
    try {
      await invoke("start_server");
    } catch (e) {
      console.error("start_server failed", e);
    }
  }

  async function stopServer() {
    try {
      await invoke("stop_server");
    } catch (e) {
      console.error("stop_server failed", e);
    }
  }

  async function setVolume(value: number) {
    volume.value = value;
    try {
      await invoke("set_volume", { value });
    } catch (e) {
      console.error("set_volume failed", e);
    }
  }

  async function toggleMute() {
    muted.value = !muted.value;
    try {
      await invoke("toggle_mute");
    } catch (e) {
      console.error("toggle_mute failed", e);
    }
  }

  async function setBufferSize(ms: number) {
    bufferSize.value = ms;
    try {
      await invoke("set_buffer_size", { ms });
    } catch (e) {
      console.error("set_buffer_size failed", e);
    }
  }

  return {
    connection,
    latency,
    packetLoss,
    bufferSize,
    volume,
    muted,
    deviceName,
    statusText,
    statusColor,
    init,
    cleanup,
    startServer,
    stopServer,
    setVolume,
    toggleMute,
    setBufferSize,
  };
});
