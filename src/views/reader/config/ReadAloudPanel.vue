<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { useRouter } from "vue-router";
import type { ReaderSnapshot } from "@/api/bindings";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import { useReadAloud } from "@/composables/useReadAloud";
import Icon from "@/components/android/Icon.vue";

const props = defineProps<{
  modelValue: boolean;
  snapshot: ReaderSnapshot | null;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  "page-end": [];
  finished: [];
  position: [charPos: number];
  "open-config": [];
  "show-menu": [];
  "playing-change": [boolean];
  backstage: [];
}>();

const router = useRouter();
const prefs = useReaderPrefsStore();
const timerMinutes = ref(prefs.timerMinutes);
const timerSelector = ref(false);
const TIMES = [0, 5, 10, 15, 30, 60, 90, 180];

const rateProgress = computed(() =>
  Math.min(45, Math.max(0, Math.round(prefs.readAloudRate * 10 - 5))),
);
const rateLabel = computed(() => ((rateProgress.value + 5) / 10).toFixed(1));

const pageStart = computed(() => {
  const snap = props.snapshot;
  if (!snap) return 0;
  return snap.pages[snap.pageIndex]?.startChar ?? 0;
});
const pageEnd = computed(() => {
  const snap = props.snapshot;
  if (!snap) return 0;
  return snap.pages[snap.pageIndex]?.endChar ?? 0;
});

let continueNextChapter = false;

const aloud = useReadAloud({
  getParagraphs: () => props.snapshot?.chapterParagraphs ?? [],
  getPageStart: () => pageStart.value,
  getPageEnd: () => pageEnd.value,
  onPageEnd: () => emit("page-end"),
  onFinished: () => {
    continueNextChapter = true;
    emit("finished");
  },
  onPosition: (pos) => emit("position", pos),
});

watch(
  () => aloud.speaking.value && !aloud.paused.value,
  (playing) => emit("playing-change", playing),
);

aloud.rate.value = prefs.readAloudRate;
aloud.timerMinutes.value = timerMinutes.value;

function stopAll() {
  continueNextChapter = false;
  aloud.stop();
  emit("position", -1);
}

function close() {
  stopAll();
  emit("update:modelValue", false);
}

function togglePlay() {
  if (!aloud.speaking.value) {
    aloud.start();
  } else if (aloud.paused.value) {
    aloud.resume();
  } else {
    aloud.pause();
  }
}

function updateRateProgress(value: number) {
  const next = Math.min(45, Math.max(0, value));
  prefs.patch({ readAloudRate: (next + 5) / 10 });
  aloud.rate.value = (next + 5) / 10;
}

function updateTimer(value: number) {
  timerMinutes.value = value;
  aloud.setTimer(value);
  prefs.patch({ timerMinutes: value });
}

function chooseTimer(minutes: number) {
  updateTimer(minutes);
  timerSelector.value = false;
}

function saveTimerDefault() {
  prefs.patch({ timerMinutes: timerMinutes.value });
}

function prevChapter() {
  if (props.snapshot && props.snapshot.chapterIndex > 0) {
    emit("page-end");
    router.push({
      path: "/reader",
      query: {
        bookUrl: props.snapshot.book.bookUrl,
        chapter: String(props.snapshot.chapterIndex - 1),
      },
    });
  }
}

function nextChapter() {
  if (props.snapshot && props.snapshot.chapterIndex < props.snapshot.totalChapters - 1) {
    emit("page-end");
    router.push({
      path: "/reader",
      query: {
        bookUrl: props.snapshot.book.bookUrl,
        chapter: String(props.snapshot.chapterIndex + 1),
      },
    });
  }
}

function goToc() {
  close();
  if (props.snapshot) {
    router.push({
      path: "/toc",
      query: { bookUrl: props.snapshot.book.bookUrl },
    });
  }
}

watch(
  () => props.modelValue,
  async (open) => {
    if (open) {
      await aloud.loadEngines();
    } else {
      timerSelector.value = false;
      stopAll();
    }
  },
);

watch(
  () => props.snapshot?.chapterIndex,
  () => {
    if (!props.modelValue) return;
    if (continueNextChapter) {
      continueNextChapter = false;
      window.setTimeout(() => aloud.start(), 120);
    } else if (aloud.speaking.value) {
      aloud.reloadAndContinue();
    }
  },
);

onBeforeUnmount(() => aloud.stop());
</script>

<template>
  <div v-if="modelValue" class="mask" @click.self="close">
    <!-- dialog_read_aloud.xml 底部面板 -->
    <div class="panel" data-id="root_view">
      <div class="play-row">
        <button class="chapter-btn" data-id="tv_pre" @click="prevChapter">上一章</button>
        <button class="chapter-btn" data-id="tv_next" @click="nextChapter">下一章</button>
        <span class="spacer" />
        <button class="ctrl" aria-label="上一句" data-id="iv_play_prev" @click="aloud.skip(-1)">
          <Icon name="ic_skip_previous" :size="26" />
        </button>
        <button class="ctrl" aria-label="播放/暂停" data-id="iv_play_pause" @click="togglePlay">
          <Icon :name="aloud.speaking.value && !aloud.paused.value ? 'ic_pause_24dp' : 'ic_play_24dp'" :size="26" />
        </button>
        <button class="ctrl" aria-label="停止" data-id="iv_stop" @click="stopAll">
          <Icon name="ic_stop_black_24dp" :size="26" />
        </button>
        <button class="ctrl" aria-label="下一句" data-id="iv_play_next" @click="aloud.skip(1)">
          <Icon name="ic_skip_next" :size="26" />
        </button>
      </div>

      <div class="timer-row">
        <button class="ctrl" aria-label="保存定时" data-id="iv_timer" @click="saveTimerDefault">
          <Icon name="ic_time_add_24dp" :size="24" />
        </button>
        <input
          type="range"
          data-id="seek_timer"
          min="0"
          max="180"
          step="1"
          :value="timerMinutes"
          @input="updateTimer(Number(($event.target as HTMLInputElement).value))"
        />
        <button class="timer-text" data-id="tv_timer" @click="timerSelector = !timerSelector">
          {{ timerMinutes }}分
        </button>
      </div>
      <div v-if="timerSelector" class="timer-selector">
        <button
          v-for="minutes in TIMES"
          :key="minutes"
          :class="{ active: timerMinutes === minutes }"
          @click="chooseTimer(minutes)"
        >
          {{ minutes }} 分钟
        </button>
      </div>

      <div class="rate-row" data-id="ll_tts_SpeechRate">
        <span class="rate-label" data-id="tv_tts_speed">语速</span>
        <span
          class="rate-label"
          data-id="tv_tts_speed_value"
          v-show="!prefs.ttsFollowSystem"
        >
          {{ rateLabel }}
        </span>
        <label class="follow-sys">
          跟随系统
          <input
            type="checkbox"
            data-id="cb_tts_follow_sys"
            :checked="prefs.ttsFollowSystem"
            @change="prefs.patch({ ttsFollowSystem: ($event.target as HTMLInputElement).checked })"
          />
        </label>
      </div>
      <div class="rate-seek">
        <button
          class="ctrl"
          aria-label="减慢"
          data-id="iv_tts_speech_reduce"
          :disabled="prefs.ttsFollowSystem"
          @click="updateRateProgress(rateProgress - 1)"
        >
          <Icon name="ic_reduce" :size="24" />
        </button>
        <input
          type="range"
          data-id="seek_tts_speechRate"
          min="0"
          max="45"
          step="1"
          :value="rateProgress"
          :disabled="prefs.ttsFollowSystem"
          @input="updateRateProgress(Number(($event.target as HTMLInputElement).value))"
        />
        <button
          class="ctrl"
          aria-label="加快"
          data-id="iv_tts_speech_add"
          :disabled="prefs.ttsFollowSystem"
          @click="updateRateProgress(rateProgress + 1)"
        >
          <Icon name="ic_add" :size="24" />
        </button>
      </div>

      <div class="bottom-btns">
        <button data-id="ll_catalog" @click="goToc">
          <Icon name="ic_toc" :size="22" data-id="iv_catalog" />
          <span data-id="tv_catalog">目录</span>
        </button>
        <button data-id="ll_main_menu" @click="emit('show-menu')">
          <Icon name="ic_menu" :size="22" data-id="iv_main_menu" />
          <span data-id="tv_main_menu">主菜单</span>
        </button>
        <button data-id="ll_to_backstage" @click="emit('backstage')">
          <Icon name="ic_visibility_off" :size="22" data-id="iv_to_backstage" />
          <span data-id="tv_to_backstage">后台</span>
        </button>
        <button data-id="ll_setting" @click="emit('open-config')">
          <Icon name="ic_settings" :size="22" data-id="iv_setting" />
          <span data-id="tv_setting">设置</span>
        </button>
      </div>

      <p v-if="aloud.error.value" class="error">{{ aloud.error.value }}</p>
    </div>
  </div>
</template>

<style scoped>
.mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  z-index: 40;
}
.panel {
  width: min(100%, 720px);
  background: var(--ac-background-menu);
  padding: 6px 16px calc(6px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.play-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.spacer {
  flex: 1;
  min-width: 8px;
}
.chapter-btn {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 14px;
  padding: 10px 10px;
  cursor: pointer;
  white-space: nowrap;
}
.ctrl {
  width: 36px;
  height: 36px;
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.ctrl:disabled {
  opacity: 0.4;
  cursor: default;
}
.timer-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}
.timer-row input {
  flex: 1;
  min-width: 0;
}
.timer-text {
  flex: none;
  border: none;
  background: transparent;
  font-size: 13px;
  color: var(--ac-primary-text);
  cursor: pointer;
  padding: 6px 2px;
}
.timer-selector {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 4px 0 8px;
}
.timer-selector button {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 6px 12px;
  font-size: 13px;
  cursor: pointer;
}
.timer-selector button.active {
  border-color: var(--ac-primary);
  color: var(--ac-primary);
}
.rate-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0 2px;
}
.rate-label {
  font-size: 14px;
  color: var(--ac-primary-text);
}
.follow-sys {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.rate-seek {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 6px;
}
.rate-seek input {
  flex: 1;
  min-width: 0;
}
.bottom-btns {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-top: 8px;
  padding: 0 12px;
}
.bottom-btns button {
  width: 60px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 12px;
  padding-bottom: 7px;
  cursor: pointer;
}
.error {
  margin: 0;
  padding: 4px 0;
  font-size: 12px;
  color: var(--ac-error);
}
</style>
