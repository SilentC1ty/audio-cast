<script setup lang="ts">
import { useAudioStore } from "../stores/audio";

const audio = useAudioStore();

function onVolumeInput(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  audio.setVolume(val);
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <label class="text-xs text-neutral-400">音量</label>
      <span class="text-xs text-neutral-500">{{ Math.round(audio.volume * 100) }}%</span>
    </div>
    <div class="flex items-center gap-2">
      <input
        type="range"
        min="0"
        max="1.5"
        step="0.05"
        :value="audio.volume"
        @input="onVolumeInput"
        class="flex-1 accent-green-500 h-1.5"
      />
      <button
        @click="audio.toggleMute()"
        class="w-8 h-8 flex items-center justify-center rounded-md transition-colors"
        :class="audio.muted ? 'bg-red-500/20 text-red-400' : 'bg-neutral-700 text-neutral-300 hover:bg-neutral-600'"
      >
        <svg v-if="audio.muted" xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 5L6 9H2v6h4l5 4V5z" /><line x1="23" y1="9" x2="17" y2="15" /><line x1="17" y1="9" x2="23" y2="15" />
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 5L6 9H2v6h4l5 4V5z" /><path d="M19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.07" />
        </svg>
      </button>
    </div>
  </div>
</template>
