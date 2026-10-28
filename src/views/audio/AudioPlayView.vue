<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const router = useRouter();
const message = ref("");
const progress = ref(0);
const speed = ref(1);

function notReady() {
  message.value = "音频播放功能暂未接入";
}
</script>

<template>
  <div class="audio-page">
    <div class="bg-mask" />
    <TitleBar title="音频播放" dark @back="router.back()" />
    <main class="content">
      <p v-if="message" class="message">{{ message }}</p>
      <button class="timer" @click="notReady">
        <Icon name="ic_timer_black_24dp" :size="18" /> 定时
      </button>
      <div class="cover">
        <CoverImage name="音频" :width="260" :height="260" :radius="130" />
      </div>
      <p class="chapter">暂无音频章节</p>
      <div class="progress-row">
        <span>00:00</span>
        <input v-model.number="progress" type="range" min="0" max="100" @input="notReady" />
        <span>00:00</span>
      </div>
      <div class="controls">
        <button @click="notReady"><Icon name="ic_timer_black_24dp" :size="22" /></button>
        <button @click="notReady"><Icon name="ic_fast_rewind" :size="22" /></button>
        <button @click="notReady"><Icon name="ic_skip_previous" :size="22" /></button>
        <button class="play" @click="notReady"><Icon name="ic_play_24dp" :size="28" /></button>
        <button @click="notReady"><Icon name="ic_skip_next" :size="22" /></button>
        <button @click="notReady"><Icon name="ic_stop_black_24dp" :size="22" /></button>
        <button @click="notReady">{{ speed }}x</button>
      </div>
    </main>
  </div>
</template>

<style scoped>
.audio-page {
  position: relative;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #111;
}
.bg-mask {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.35), rgba(0, 0, 0, 0.88));
}
:deep(.title-bar) {
  position: relative;
  z-index: 1;
}
.content {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 16px calc(8px + env(safe-area-inset-bottom));
  color: #fff;
}
.message {
  margin: 4px 0;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
}
.timer {
  align-self: flex-start;
  border: none;
  background: rgba(255, 255, 255, 0.18);
  color: #fff;
  border-radius: 14px;
  padding: 4px 10px;
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}
.cover {
  margin-top: 8vh;
  border: 2px solid var(--ac-accent);
  border-radius: 50%;
  overflow: hidden;
}
.chapter {
  margin: 18px 0 8px;
  font-size: 15px;
}
.progress-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.progress-row input {
  flex: 1;
  accent-color: #fff;
}
.controls {
  margin-top: 24px;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.controls button {
  width: 42px;
  height: 42px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 12px;
}
.controls button:hover {
  background: rgba(255, 255, 255, 0.12);
}
.controls .play {
  width: 62px;
  height: 62px;
  background: #fff;
  color: #111;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}
</style>
