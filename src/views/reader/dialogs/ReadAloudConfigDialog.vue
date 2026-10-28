<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import prefConfig from "@/design/prefs/pref_config_aloud.json";
import PrefTree, { type PrefNode } from "@/components/android/PrefTree.vue";
import AndroidDialog from "./AndroidDialog.vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import { useReaderSettingsStore, type PrefValue } from "@/stores/readerSettings";
import { httpTtsEngines } from "@/api/rules";
import type { HttpTts } from "@/api/bindings";
import SpeakEngineDialog from "./SpeakEngineDialog.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const prefs = useReaderPrefsStore();
const settings = useReaderSettingsStore();
const nodes = computed(() => prefConfig.children as unknown as PrefNode[]);
const engines = ref<HttpTts[]>([]);
const engineVisible = ref(false);
const toast = ref("");
let toastTimer: number | undefined;

const engineLabel = computed(() => {
  if (!prefs.ttsEngine) return "系统 TTS";
  const engine = engines.value.find((item) => String(item.id) === prefs.ttsEngine);
  return engine?.name ?? "系统 TTS";
});
const summaries = computed<Record<string, string>>(() => ({
  appTtsEngine: engineLabel.value,
}));

const UNSUPPORTED = new Set([
  "pauseReadAloudWhilePhoneCalls",
  "readAloudWakeLock",
  "systemMediaControlCompatibilityChange",
  "mediaButtonPerNext",
  "sysTtsConfig",
]);

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    settings.set("ignoreAudioFocus", prefs.ignoreAudioFocus);
    settings.set("readAloudByPage", prefs.readAloudByPage);
    settings.set("streamReadAloudAudio", prefs.streamReadAloudAudio);
  },
  { immediate: true },
);

function showToast(message: string) {
  toast.value = message;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (toast.value = ""), 1600);
}

function onChange(key: string, value: PrefValue) {
  settings.set(key, value);
  const bool = value === true || value === "true";
  if (key === "ignoreAudioFocus") prefs.patch({ ignoreAudioFocus: bool });
  else if (key === "readAloudByPage") prefs.patch({ readAloudByPage: bool });
  else if (key === "streamReadAloudAudio") prefs.patch({ streamReadAloudAudio: bool });
  else if (UNSUPPORTED.has(key)) showToast("桌面版暂不支持");
}

function onAction(node: PrefNode) {
  if (node.key === "appTtsEngine") {
    engineVisible.value = true;
    return;
  }
  showToast("桌面版暂不支持");
}

onMounted(async () => {
  try {
    engines.value = await httpTtsEngines();
  } catch {
    engines.value = [];
  }
});
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    :show-header="false"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="aloud-config">
      <PrefTree
        :nodes="nodes"
        :model="settings.values"
        :summaries="summaries"
        @change="onChange"
        @action="onAction"
      />
      <p v-if="toast" class="toast">{{ toast }}</p>
    </div>
    <SpeakEngineDialog v-model="engineVisible" />
  </AndroidDialog>
</template>

<style scoped>
.aloud-config {
  display: flex;
  flex-direction: column;
  max-height: 70vh;
  overflow-y: auto;
}
.toast {
  margin: 0;
  padding: 8px 16px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
</style>
