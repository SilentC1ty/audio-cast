<script setup lang="ts">
import { useAudioStore } from "../stores/audio";

const audio = useAudioStore();
</script>

<template>
  <div class="flex items-center gap-3">
    <div :class="['w-3 h-3 rounded-full', audio.statusColor]" />
    <div class="flex flex-col">
      <span class="text-sm font-medium text-white">{{ audio.statusText }}</span>
      <span v-if="audio.deviceName" class="text-xs text-neutral-400">
        设备: {{ audio.deviceName }}
      </span>
    </div>
  </div>
  <div v-if="audio.connection === 'connected' || audio.connection === 'streaming'" class="flex gap-4 text-xs text-neutral-400">
    <span>延迟: {{ audio.latency }}ms</span>
    <span>丢包: {{ audio.packetLoss.toFixed(1) }}%</span>
  </div>
</template>
