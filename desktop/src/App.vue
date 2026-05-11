<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const status = ref("idle");
const bufferSize = ref(60);
const volume = ref(80);

async function fetchStatus() {
  try {
    const res = await invoke<string>("get_status");
    const data = JSON.parse(res);
    status.value = data.status;
  } catch (e) {
    status.value = "disconnected";
  }
}

async function setBufferSize(val: number) {
  bufferSize.value = val;
  try {
    await invoke<string>("set_buffer_size", { bufferMs: val });
  } catch (e) {
    console.error("set_buffer_size failed", e);
  }
}

onMounted(() => {
  fetchStatus();
});
</script>

<template>
  <div class="h-screen w-screen bg-neutral-900 text-white select-none drag">
    <div class="flex flex-col h-full p-4 gap-4">
      <!-- 标题 -->
      <div class="flex items-center gap-2">
        <div
          class="w-3 h-3 rounded-full"
          :class="{
            'bg-green-500': status === 'streaming',
            'bg-yellow-500': status === 'idle',
            'bg-red-500': status === 'disconnected',
          }"
        />
        <span class="text-sm font-medium">AudioCast</span>
        <span class="text-xs text-neutral-500">{{ status }}</span>
      </div>

      <!-- 音量控制 -->
      <div class="flex flex-col gap-1">
        <label class="text-xs text-neutral-400">音量</label>
        <input
          type="range"
          min="0"
          max="100"
          :value="volume"
          @input="volume = Number(($event.target as HTMLInputElement).value)"
          class="w-full accent-green-500"
        />
      </div>

      <!-- 缓冲区调节 -->
      <div class="flex flex-col gap-1">
        <label class="text-xs text-neutral-400">
          缓冲区: {{ bufferSize }}ms
        </label>
        <input
          type="range"
          min="20"
          max="200"
          step="10"
          :value="bufferSize"
          @input="setBufferSize(Number(($event.target as HTMLInputElement).value))"
          class="w-full accent-blue-500"
        />
        <div class="flex justify-between text-[10px] text-neutral-500">
          <span>低延迟</span>
          <span>抗干扰</span>
        </div>
      </div>

      <!-- 信息 -->
      <div class="mt-auto text-center text-xs text-neutral-600">
        AudioCast v0.1.0
      </div>
    </div>
  </div>
</template>
