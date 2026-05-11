<script setup lang="ts">
import { useAudioStore } from "../stores/audio";

const audio = useAudioStore();

function onBufferInput(e: Event) {
  const val = parseInt((e.target as HTMLInputElement).value);
  audio.setBufferSize(val);
}
</script>

<template>
  <div class="flex flex-col gap-1">
    <div class="flex items-center justify-between">
      <label class="text-xs text-neutral-400">缓冲区</label>
      <span class="text-xs text-neutral-500">{{ audio.bufferSize }}ms</span>
    </div>
    <input
      type="range"
      min="60"
      max="120"
      step="10"
      :value="audio.bufferSize"
      @input="onBufferInput"
      class="w-full accent-blue-500 h-1.5"
    />
    <div class="flex justify-between text-[10px] text-neutral-500">
      <span>低延迟</span>
      <span>抗干扰</span>
    </div>
  </div>
</template>
