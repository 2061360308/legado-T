<script setup lang="ts">
import { computed, ref, watch } from "vue";
import prefConfig from "@/design/prefs/pref_config_read.json";
import PrefTree, { type PrefNode } from "@/components/android/PrefTree.vue";
import AndroidDialog from "./AndroidDialog.vue";
import PreferenceListDialog from "./PreferenceListDialog.vue";
import NumberPickerDialog from "./NumberPickerDialog.vue";
import PageKeyDialog from "./PageKeyDialog.vue";
import ClickActionDialog from "./ClickActionDialog.vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import { useReaderSettingsStore, type PrefValue } from "@/stores/readerSettings";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean]; apply: [] }>();

const prefs = useReaderPrefsStore();
const settings = useReaderSettingsStore();
const nodes = computed(() => prefConfig.children as unknown as PrefNode[]);
const listNode = ref<PrefNode | null>(null);
const listVisible = ref(false);
const numberVisible = ref(false);
const pageKeyVisible = ref(false);
const clickVisible = ref(false);
const toast = ref("");
let toastTimer: number | undefined;

const UNSUPPORTED = new Set([
  "autoChangeSource",
]);

const listEntries = computed(() => listNode.value?.entries ?? []);
const listValues = computed(() => listNode.value?.entryValues ?? []);
const listValue = computed(() => {
  const node = listNode.value;
  if (!node) return "";
  const value = settings.values[node.key];
  return value === undefined ? node.defaultValue : String(value);
});

watch(
  () => props.modelValue,
  (open) => {
    if (open) syncFromPrefs();
  },
  { immediate: true },
);

function syncFromPrefs() {
  settings.set("useZhLayout", prefs.useZhLayout);
  settings.set("textFullJustify", prefs.fullJustify);
  settings.set("textBottomJustify", prefs.bottomJustify);
  settings.set("showBrightnessView", prefs.showBrightness);
  settings.set("showReadTitleAddition", prefs.showTitleAddition);
  settings.set("mouseWheelPage", prefs.mouseWheelPage);
  settings.set("pageTouchSlop", String(prefs.pageTouchSlop));
}

function showToast(message: string) {
  toast.value = message;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (toast.value = ""), 1600);
}

function applyKey(key: string, value: PrefValue) {
  const bool = value === true || value === "true";
  switch (key) {
    case "useZhLayout":
      prefs.patch({ useZhLayout: bool });
      break;
    case "textFullJustify":
      prefs.patch({ fullJustify: bool });
      break;
    case "textBottomJustify":
      prefs.patch({ bottomJustify: bool });
      break;
    case "showBrightnessView":
      prefs.patch({ showBrightness: bool });
      break;
    case "showReadTitleAddition":
      prefs.patch({ showTitleAddition: bool });
      break;
    case "mouseWheelPage":
      prefs.patch({ mouseWheelPage: bool });
      break;
    case "pageTouchSlop":
      prefs.patch({ pageTouchSlop: Number(value) || prefs.pageTouchSlop });
      break;
    default:
      if (UNSUPPORTED.has(key)) showToast("桌面版暂不支持");
  }
  emit("apply");
}

function onChange(key: string, value: PrefValue) {
  settings.set(key, value);
  applyKey(key, value);
}

function onAction(node: PrefNode) {
  if (node.entries?.length) {
    listNode.value = node;
    listVisible.value = true;
    return;
  }
  if (node.key === "pageTouchSlop") {
    numberVisible.value = true;
    return;
  }
  if (node.key === "clickRegionalConfig") {
    clickVisible.value = true;
    return;
  }
  if (node.key === "customPageKey") {
    pageKeyVisible.value = true;
    return;
  }
  showToast("桌面版暂不支持");
}

function onListConfirm(value: string) {
  const node = listNode.value;
  if (!node) return;
  settings.set(node.key, value);
  applyKey(node.key, value);
}

function setSlop(value: number) {
  settings.set("pageTouchSlop", String(value));
  prefs.patch({ pageTouchSlop: value });
  emit("apply");
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="sheet"
    height="360px"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="more-config">
      <PrefTree
        :nodes="nodes"
        :model="settings.values"
        @change="onChange"
        @action="onAction"
      />

      <p v-if="toast" class="toast">{{ toast }}</p>
    </div>

    <PreferenceListDialog
      v-model="listVisible"
      :title="listNode?.title ?? ''"
      :entries="listEntries"
      :entry-values="listValues"
      :value="listValue"
      @confirm="onListConfirm"
    />

    <NumberPickerDialog
      v-model="numberVisible"
      title="滑动翻页阈值（0 = 系统默认值）"
      :min="0"
      :max="9999"
      :value="prefs.pageTouchSlop"
      @confirm="setSlop"
    />

    <PageKeyDialog v-model="pageKeyVisible" />
    <ClickActionDialog v-model="clickVisible" />
  </AndroidDialog>
</template>

<style scoped>
.more-config {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
}
.toast {
  margin: 0;
  padding: 8px 16px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
</style>
