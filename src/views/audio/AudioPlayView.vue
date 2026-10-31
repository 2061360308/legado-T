<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const router = useRouter();
const message = ref("");
const audioRef = ref<HTMLAudioElement | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const fileName = ref("");
const objectUrl = ref("");
const playing = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const speed = ref(1);
const timerMinutes = ref(0);
const coverSize = ref(260);
let timer: ReturnType<typeof setTimeout> | null = null;

function updateCoverSize() {
  coverSize.value = Math.min(260, Math.max(150, Math.floor(window.innerHeight * 0.36)));
}

onMounted(() => {
  updateCoverSize();
  window.addEventListener("resize", updateCoverSize);
});

const speeds = [0.5, 0.75, 1, 1.25, 1.5, 2];
const progress = computed({
  get: () => (duration.value ? (currentTime.value / duration.value) * 100 : 0),
  set: (value: number) => {
    if (!audioRef.value || !duration.value) return;
    audioRef.value.currentTime = (value / 100) * duration.value;
  },
});

function fmt(seconds: number) {
  if (!Number.isFinite(seconds)) return "00:00";
  const total = Math.max(0, Math.floor(seconds));
  return `${String(Math.floor(total / 60)).padStart(2, "0")}:${String(total % 60).padStart(2, "0")}`;
}

function pickFile() {
  fileInput.value?.click();
}

function onFile(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  if (objectUrl.value) URL.revokeObjectURL(objectUrl.value);
  objectUrl.value = URL.createObjectURL(file);
  fileName.value = file.name.replace(/\.[^.]+$/, "");
  currentTime.value = 0;
  duration.value = 0;
  playing.value = false;
  message.value = "";
  requestAnimationFrame(() => {
    if (audioRef.value) audioRef.value.src = objectUrl.value;
  });
}

function togglePlay() {
  const audio = audioRef.value;
  if (!audio || !objectUrl.value) {
    pickFile();
    return;
  }
  if (audio.paused) void audio.play();
  else audio.pause();
}

function skip(seconds: number) {
  const audio = audioRef.value;
  if (!audio) return;
  audio.currentTime = Math.max(0, Math.min(duration.value, audio.currentTime + seconds));
}

function cycleSpeed() {
  const index = speeds.indexOf(speed.value);
  speed.value = speeds[(index + 1) % speeds.length];
  if (audioRef.value) audioRef.value.playbackRate = speed.value;
}

function cycleTimer() {
  const options = [0, 15, 30, 60];
  timerMinutes.value = options[(options.indexOf(timerMinutes.value) + 1) % options.length];
  if (timer) clearTimeout(timer);
  timer = null;
  if (timerMinutes.value > 0) {
    timer = setTimeout(() => {
      audioRef.value?.pause();
      message.value = "定时播放已结束";
      timerMinutes.value = 0;
    }, timerMinutes.value * 60 * 1000);
  }
  message.value = timerMinutes.value ? `定时 ${timerMinutes.value} 分钟` : "定时已关闭";
}

function onTimeUpdate() {
  currentTime.value = audioRef.value?.currentTime ?? 0;
}

function onLoaded() {
  duration.value = audioRef.value?.duration ?? 0;
}

function onEnded() {
  playing.value = false;
}

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateCoverSize);
  if (timer) clearTimeout(timer);
  if (objectUrl.value) URL.revokeObjectURL(objectUrl.value);
});
</script>

<template>
  <div class="audio-page">
    <div class="bg-mask" />
    <TitleBar title="音频播放" dark @back="router.back()">
      <template #actions>
        <button class="speed-badge" @click="cycleSpeed">{{ speed }}x</button>
        <button aria-label="章节列表" @click="message = '暂无章节列表'">
          <Icon name="ic_chapter_list" :size="24" />
        </button>
        <button aria-label="更多" @click="message = '更多功能暂未接入'">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>
    <main class="content">
      <p v-if="message" class="message">{{ message }}</p>
      <button class="timer" @click="cycleTimer">
        <Icon name="ic_timer_black_24dp" :size="18" />
        {{ timerMinutes ? `${timerMinutes} 分钟` : "定时" }}
      </button>
      <div class="cover">
        <CoverImage
          name="音频"
          :width="coverSize"
          :height="coverSize"
          :radius="coverSize / 2"
        />
      </div>
      <p class="chapter">{{ fileName || "选择本地音频文件" }}</p>
      <button class="choose" @click="pickFile">{{ fileName ? "更换音频" : "选择音频" }}</button>
      <input ref="fileInput" class="file-input" type="file" accept="audio/*" @change="onFile" />
      <audio
        ref="audioRef"
        @timeupdate="onTimeUpdate"
        @loadedmetadata="onLoaded"
        @play="playing = true"
        @pause="playing = false"
        @ended="onEnded"
      />
      <div class="progress-row">
        <span>{{ fmt(currentTime) }}</span>
        <input v-model.number="progress" type="range" min="0" max="100" step="0.1" />
        <span>{{ fmt(duration) }}</span>
      </div>
      <div class="controls">
        <button @click="skip(-30)"><Icon name="ic_fast_rewind" :size="22" /></button>
        <button @click="skip(-15)"><Icon name="ic_skip_previous" :size="22" /></button>
        <button class="play" @click="togglePlay">
          <Icon :name="playing ? 'ic_pause_24dp' : 'ic_play_24dp'" :size="28" />
        </button>
        <button @click="skip(15)"><Icon name="ic_skip_next" :size="22" /></button>
        <button @click="skip(30)"><Icon name="ic_play_mode_list_end_stop" :size="22" /></button>
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
.speed-badge {
  min-width: 38px;
  border: none;
  background: rgba(255, 255, 255, 0.18);
  color: #fff;
  border-radius: 12px;
  padding: 3px 8px;
  font-size: 12px;
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
  cursor: pointer;
}
.cover {
  margin-top: 2vh;
  border: 2px solid var(--ac-accent);
  border-radius: 50%;
  overflow: hidden;
}
.chapter {
  margin: 2vh 0 4px;
  font-size: 15px;
  max-width: 90%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.choose {
  border: 1px solid rgba(255, 255, 255, 0.4);
  background: transparent;
  color: #fff;
  border-radius: 14px;
  padding: 5px 14px;
  font-size: 12px;
  cursor: pointer;
}
.file-input {
  display: none;
}
audio {
  display: none;
}
.progress-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  margin-top: 2vh;
}
.progress-row input {
  flex: 1;
  accent-color: #fff;
}
.controls {
  margin-top: 2vh;
  margin-bottom: 2vh;
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
