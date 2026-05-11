<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useAudioStore } from "./stores/audio";
import StatusIndicator from "./components/StatusIndicator.vue";
import VolumeControl from "./components/VolumeControl.vue";
import BufferControl from "./components/BufferControl.vue";

const audio = useAudioStore();

onMounted(() => {
  audio.init();
});

onUnmounted(() => {
  audio.cleanup();
});
</script>

<template>
  <div class="h-screen w-screen bg-neutral-900 text-white select-none flex flex-col">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-neutral-800">
      <StatusIndicator />
    </div>

    <!-- 控制区 -->
    <div class="flex-1 flex flex-col gap-5 p-4">
      <VolumeControl />
      <BufferControl />
    </div>

    <!-- 操作栏 -->
    <div class="px-4 py-3 border-t border-neutral-800 flex gap-2">
      <button
        v-if="audio.connection === 'idle' || audio.connection === 'disconnected'"
        @click="audio.startServer()"
        class="flex-1 py-2 rounded-lg bg-green-600 hover:bg-green-500 text-sm font-medium transition-colors"
      >
        启动服务
      </button>
      <button
        v-else
        @click="audio.stopServer()"
        class="flex-1 py-2 rounded-lg bg-red-600 hover:bg-red-500 text-sm font-medium transition-colors"
      >
        停止服务
      </button>
    </div>

    <!-- 版本 -->
    <div class="pb-2 text-center text-[10px] text-neutral-600">
      AudioCast v0.1.0
    </div>
  </div>
</template>
