<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { bookGet, chapterList } from "@/core/application/books";
import { bookSourceResource } from "@/core/application/bookSources";
import { readerSaveProgress } from "@/core/application/reader";
import type { Book, BookChapter } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const route = useRoute();
const router = useRouter();
const bookUrl = computed(() => String(route.query.bookUrl || ""));
const book = ref<Book | null>(null);
const chapters = ref<BookChapter[]>([]);
const chapterIndex = ref(0);
const onlineUrl = ref("");
const message = ref("");
const audioRef = ref<HTMLAudioElement | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const fileName = ref("");
const objectUrl = ref("");
const onlineMode = computed(() => Boolean(bookUrl.value));
const audioSrc = computed(() => onlineUrl.value || objectUrl.value);
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
  void loadOnline();
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

function readableIndex(start: number): number {
  if (!chapters.value.length) return start;
  let index = Math.max(0, Math.min(chapters.value.length - 1, start));
  for (let count = 0; count < chapters.value.length; count += 1) {
    if (!chapters.value[index]?.isVolume) return chapters.value[index].index;
    index = (index + 1) % chapters.value.length;
  }
  return start;
}

async function loadOnline() {
  if (!bookUrl.value) return;
  try {
    book.value = await bookGet(bookUrl.value);
    chapters.value = await chapterList(bookUrl.value);
    await loadChapter(readableIndex(book.value?.durChapterIndex ?? 0));
  } catch (error) {
    message.value = String(error);
  }
}

async function loadChapter(index: number) {
  const chapter = chapters.value.find((item) => item.index === index);
  if (!chapter) return;
  message.value = "";
  onlineUrl.value = "";
  chapterIndex.value = index;
  fileName.value = chapter.title;
  currentTime.value = 0;
  duration.value = 0;
  playing.value = false;
  try {
    const resource = await bookSourceResource(bookUrl.value, index);
    onlineUrl.value = resource.url;
    fileName.value = resource.title || chapter.title;
    await readerSaveProgress(
      bookUrl.value,
      index,
      0,
      fileName.value,
    ).catch(() => undefined);
  } catch (error) {
    message.value = String(error);
  }
}

async function changeOnlineChapter(delta: number) {
  const position = chapters.value.findIndex((item) => item.index === chapterIndex.value);
  const target = position + delta;
  if (target < 0 || target >= chapters.value.length) return;
  await loadChapter(readableIndex(target));
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
  if (!audio || !audioSrc.value) {
    if (!onlineMode.value) pickFile();
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
  if (onlineMode.value) void changeOnlineChapter(1);
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
    <TitleBar :title="book?.name || '音频播放'" dark @back="router.back()">
      <template #actions>
        <button class="speed-badge" @click="cycleSpeed">{{ speed }}x</button>
        <button
          aria-label="章节列表"
          @click="onlineMode ? router.push({ path: '/toc', query: { bookUrl } }) : (message = '暂无章节列表')"
        >
          <Icon name="ic_chapter_list" :size="24" />
        </button>
        <button aria-label="下一章" :disabled="!onlineMode" @click="changeOnlineChapter(1)">
          <Icon name="ic_skip_next" :size="24" />
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
          :name="book?.name || '音频'"
          :author="book?.author || ''"
          :src="book?.coverUrl || null"
          :width="coverSize"
          :height="coverSize"
          :radius="coverSize / 2"
        />
      </div>
      <p class="chapter">{{ fileName || (onlineMode ? "加载音频中…" : "选择本地音频文件") }}</p>
      <button v-if="!onlineMode" class="choose" @click="pickFile">
        {{ fileName ? "更换音频" : "选择音频" }}
      </button>
      <input v-if="!onlineMode" ref="fileInput" class="file-input" type="file" accept="audio/*" @change="onFile" />
      <div v-if="onlineMode && chapters.length" class="chapter-nav">
        <button :disabled="chapterIndex <= 0" @click="changeOnlineChapter(-1)">上一章</button>
        <span>{{ chapterIndex + 1 }} / {{ chapters.length }}</span>
        <button :disabled="chapterIndex >= chapters.length - 1" @click="changeOnlineChapter(1)">下一章</button>
      </div>
      <audio
        ref="audioRef"
        :src="audioSrc || undefined"
        preload="metadata"
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
.chapter-nav {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 6px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.72);
}
.chapter-nav button {
  border: 1px solid rgba(255, 255, 255, 0.24);
  background: transparent;
  color: #fff;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
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
