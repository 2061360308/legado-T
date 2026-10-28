<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import {
  buildInfoBarStyle,
  drawInfoBar,
  measureInfoBarHeights,
  type InfoBarContext,
} from "../infoBar";
import AndroidDialog from "./AndroidDialog.vue";
import SelectorDialog from "./SelectorDialog.vue";
import DetailSeekBar from "@/components/android/DetailSeekBar.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  apply: [];
}>();

const prefs = useReaderPrefsStore();

const TIP_OPTIONS = [
  { value: "none", label: "无" },
  { value: "bookName", label: "书名" },
  { value: "chapterTitle", label: "章节名" },
  { value: "time", label: "时间" },
  { value: "battery", label: "电量" },
  { value: "batteryPercentage", label: "电量百分比" },
  { value: "page", label: "页码" },
  { value: "totalProgress", label: "总进度" },
  { value: "totalProgress1", label: "进度(章节/总章)" },
  { value: "pageAndTotal", label: "页码+总进度" },
  { value: "timeBattery", label: "时间+电量" },
  { value: "timeBatteryPercentage", label: "时间+电量百分比" },
] as const;
const TIP_VALUES: string[] = TIP_OPTIONS.map((item) => item.value);

const HEADER_MODES = [
  { value: 0, label: "状态栏显示时隐藏" },
  { value: 1, label: "显示" },
  { value: 2, label: "隐藏" },
] as const;
const FOOTER_MODES = [
  { value: 0, label: "显示" },
  { value: 1, label: "隐藏" },
] as const;

const SLOT_KEYS = [
  ["tipHeaderLeft", "页眉左"],
  ["tipHeaderMiddle", "页眉中"],
  ["tipHeaderRight", "页眉右"],
  ["tipFooterLeft", "页脚左"],
  ["tipFooterMiddle", "页脚中"],
  ["tipFooterRight", "页脚右"],
] as const;
type SlotKey = (typeof SLOT_KEYS)[number][0];

type SelectorKind =
  | ""
  | "headerMode"
  | "footerMode"
  | "tipColor"
  | "tipDividerColor"
  | `slot:${SlotKey}`;

const selectorKind = ref<SelectorKind>("");
const selectorOpen = computed({
  get: () => selectorKind.value !== "",
  set: (value: boolean) => {
    if (!value) selectorKind.value = "";
  },
});

const selectorTitle = computed(() => {
  const kind = selectorKind.value;
  if (kind === "headerMode") return "页眉显示";
  if (kind === "footerMode") return "页脚显示";
  if (kind === "tipColor") return "提示文字色";
  if (kind === "tipDividerColor") return "分隔线颜色";
  if (kind.startsWith("slot:")) {
    const key = kind.slice(5) as SlotKey;
    return SLOT_KEYS.find(([slot]) => slot === key)?.[1] ?? "";
  }
  return "";
});

const selectorItems = computed<string[]>(() => {
  const kind = selectorKind.value;
  if (kind === "headerMode") return HEADER_MODES.map((item) => item.label);
  if (kind === "footerMode") return FOOTER_MODES.map((item) => item.label);
  if (kind === "tipColor") return ["同正文色", "自定义"];
  if (kind === "tipDividerColor") return ["默认", "同正文色", "自定义"];
  if (kind.startsWith("slot:")) return TIP_OPTIONS.map((item) => item.label);
  return [];
});

const selectorSelected = computed(() => {
  const kind = selectorKind.value;
  if (kind === "headerMode") {
    return HEADER_MODES.findIndex((item) => item.value === prefs.headerMode);
  }
  if (kind === "footerMode") {
    return FOOTER_MODES.findIndex((item) => item.value === prefs.footerMode);
  }
  if (kind.startsWith("slot:")) {
    const key = kind.slice(5) as SlotKey;
    return TIP_VALUES.indexOf(prefs[key]);
  }
  return -1;
});

function update(patch: Parameters<typeof prefs.patch>[0]) {
  prefs.patch(patch);
  emit("apply");
}

function clearRepeat(targetKey: SlotKey, value: string): Record<string, string> {
  const patch: Record<string, string> = {};
  if (value === "none") return patch;
  for (const [key] of SLOT_KEYS) {
    if (key !== targetKey && prefs[key] === value) patch[key] = "none";
  }
  return patch;
}

function onSelectorSelect(index: number) {
  const kind = selectorKind.value;
  if (kind === "headerMode") {
    update({ headerMode: HEADER_MODES[index].value });
  } else if (kind === "footerMode") {
    update({ footerMode: FOOTER_MODES[index].value });
  } else if (kind.startsWith("slot:")) {
    const key = kind.slice(5) as SlotKey;
    const value = TIP_VALUES[index];
    update({ ...clearRepeat(key, value), [key]: value } as Parameters<typeof prefs.patch>[0]);
  } else if (kind === "tipColor") {
    if (index === 0) update({ tipColor: "" });
    else if (!prefs.tipColor) update({ tipColor: "#333333" });
  } else if (kind === "tipDividerColor") {
    if (index === 0) update({ tipDividerColor: "" });
    else if (index === 1) update({ tipDividerColor: "text" });
    else if (!prefs.tipDividerColor || prefs.tipDividerColor === "text") {
      update({ tipDividerColor: "#666666" });
    }
  }
  selectorKind.value = "";
}

function slotLabel(key: SlotKey): string {
  return TIP_OPTIONS.find((item) => item.value === prefs[key])?.label ?? "无";
}

function headerModeLabel(): string {
  return HEADER_MODES.find((item) => item.value === prefs.headerMode)?.label ?? "隐藏";
}

function footerModeLabel(): string {
  return FOOTER_MODES.find((item) => item.value === prefs.footerMode)?.label ?? "显示";
}

function tipColorLabel(): string {
  return prefs.tipColor || "同正文色";
}

function dividerColorLabel(): string {
  if (prefs.tipDividerColor === "text") return "同正文色";
  return prefs.tipDividerColor || "默认";
}

const titleModes = ["左对齐", "居中", "隐藏"];

const previewRef = ref<HTMLCanvasElement | null>(null);

function previewDividerColor(): string {
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue("--ac-divider")
    .trim();
  return value || "rgba(0,0,0,0.12)";
}

function previewTextColor(): string {
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue("--ac-primary-text")
    .trim();
  return value || "#222222";
}

function drawPreview() {
  const canvas = previewRef.value;
  if (!canvas || !props.modelValue) return;
  const width = Math.max(220, canvas.clientWidth || 320);
  const style = buildInfoBarStyle(prefs, {
    textColor: previewTextColor(),
    dividerColor: previewDividerColor(),
  });
  const heights = measureInfoBarHeights(style);
  const height = Math.max(72, Math.round(heights.header + heights.footer + 48));
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.floor(width * dpr));
  canvas.height = Math.max(1, Math.floor(height * dpr));
  canvas.style.height = `${height}px`;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);
  const context: InfoBarContext = {
    bookName: "示例书籍",
    chapterTitle: "第一章 示例章节",
    pageIndex: 0,
    pageSize: 12,
    chapterIndex: 0,
    chapterSize: 120,
    time: new Date().toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }),
    battery: 87,
  };
  drawInfoBar(ctx, { style, context }, width, height);
}

onMounted(() => {
  window.addEventListener("resize", drawPreview);
  nextTick(drawPreview);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", drawPreview);
});

watch(
  () => [
    props.modelValue,
    prefs.headerMode,
    prefs.footerMode,
    prefs.tipHeaderLeft,
    prefs.tipHeaderMiddle,
    prefs.tipHeaderRight,
    prefs.tipFooterLeft,
    prefs.tipFooterMiddle,
    prefs.tipFooterRight,
    prefs.tipColor,
    prefs.tipDividerColor,
    prefs.headerPaddingLeft,
    prefs.headerPaddingTop,
    prefs.headerPaddingRight,
    prefs.headerPaddingBottom,
    prefs.footerPaddingLeft,
    prefs.footerPaddingTop,
    prefs.footerPaddingRight,
    prefs.footerPaddingBottom,
    prefs.showHeaderLine,
    prefs.showFooterLine,
    prefs.titleSize,
    prefs.titleTopSpacing,
    prefs.titleBottomSpacing,
  ],
  () => nextTick(drawPreview),
);
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="信息"
    width="min(96vw, 640px)"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="info-panel">
      <section>
        <h3>正文标题</h3>
        <div class="mode-row">
          <label v-for="(label, index) in titleModes" :key="label">
            <input
              type="radio"
              :checked="prefs.titleMode === index"
              @change="update({ titleMode: index })"
            />
            {{ label }}
          </label>
        </div>
        <DetailSeekBar
          title="标题字号"
          :model-value="prefs.titleSize"
          :max="10"
          :display="(value) => `${prefs.fontSize + value}`"
          @update:model-value="(value) => update({ titleSize: value })"
        />
        <DetailSeekBar
          title="上间距"
          :model-value="prefs.titleTopSpacing"
          :max="100"
          @update:model-value="(value) => update({ titleTopSpacing: value })"
        />
        <DetailSeekBar
          title="下间距"
          :model-value="prefs.titleBottomSpacing"
          :max="100"
          @update:model-value="(value) => update({ titleBottomSpacing: value })"
        />
      </section>

      <section>
        <h3>页眉</h3>
        <button class="row" @click="selectorKind = 'headerMode'">
          <span>显示</span>
          <span class="value">{{ headerModeLabel() }} ›</span>
        </button>
        <button
          v-for="[key, label] in SLOT_KEYS.slice(0, 3)"
          :key="key"
          class="row"
          @click="selectorKind = `slot:${key}`"
        >
          <span>{{ label }}</span>
          <span class="value">{{ slotLabel(key) }} ›</span>
        </button>
      </section>

      <section>
        <h3>页脚</h3>
        <button class="row" @click="selectorKind = 'footerMode'">
          <span>显示</span>
          <span class="value">{{ footerModeLabel() }} ›</span>
        </button>
        <button
          v-for="[key, label] in SLOT_KEYS.slice(3)"
          :key="key"
          class="row"
          @click="selectorKind = `slot:${key}`"
        >
          <span>{{ label }}</span>
          <span class="value">{{ slotLabel(key) }} ›</span>
        </button>
      </section>

      <section>
        <h3>颜色</h3>
        <button class="row" @click="selectorKind = 'tipColor'">
          <span>提示文字色</span>
          <span class="value">{{ tipColorLabel() }} ›</span>
        </button>
        <label v-if="prefs.tipColor" class="color-row">
          <span>自定义颜色</span>
          <input
            type="color"
            :value="prefs.tipColor"
            @input="update({ tipColor: ($event.target as HTMLInputElement).value })"
          />
        </label>
        <button class="row" @click="selectorKind = 'tipDividerColor'">
          <span>分隔线颜色</span>
          <span class="value">{{ dividerColorLabel() }} ›</span>
        </button>
        <label
          v-if="prefs.tipDividerColor && prefs.tipDividerColor !== 'text'"
          class="color-row"
        >
          <span>自定义颜色</span>
          <input
            type="color"
            :value="prefs.tipDividerColor"
            @input="update({ tipDividerColor: ($event.target as HTMLInputElement).value })"
          />
        </label>
      </section>

      <section>
        <h3>预览</h3>
        <canvas ref="previewRef" class="info-preview" />
      </section>
    </div>

    <SelectorDialog
      v-model="selectorOpen"
      :title="selectorTitle"
      :items="selectorItems"
      :selected="selectorSelected"
      @select="onSelectorSelect"
    />
  </AndroidDialog>
</template>

<style scoped>
.info-panel {
  padding: 6px 0 12px;
}
section {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--ac-divider);
}
section:last-child {
  border-bottom: none;
}
h3 {
  margin: 8px 16px 4px;
  font-size: 13px;
  color: var(--ac-primary);
}
.mode-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  padding: 6px 16px 8px;
  font-size: 13px;
  color: var(--ac-primary-text);
}
.mode-row label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.mode-row input {
  accent-color: var(--ac-primary);
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 13px;
  padding: 9px 16px;
  text-align: left;
  cursor: pointer;
}
.row:hover {
  background: var(--ac-transparent-10);
}
.row .value {
  color: var(--ac-secondary-text);
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 60%;
}
.color-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  font-size: 13px;
  color: var(--ac-primary-text);
}
.color-row input[type="color"] {
  width: 42px;
  height: 28px;
  border: 1px solid var(--ac-divider);
  background: transparent;
  padding: 0;
}
.info-preview {
  width: calc(100% - 32px);
  margin: 4px 16px 6px;
  display: block;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
}
</style>
