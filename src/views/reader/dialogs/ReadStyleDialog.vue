<script setup lang="ts">
import { computed, ref } from "vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import { useThemeStore } from "@/stores/theme";
import { kvSet } from "@/core/application/config";
import resources from "@/design/resources.json";
import AndroidDialog from "./AndroidDialog.vue";
import DetailSeekBar from "@/components/android/DetailSeekBar.vue";
import FontSelectDialog from "./FontSelectDialog.vue";
import TipConfigDialog from "./TipConfigDialog.vue";
import SelectorDialog from "./SelectorDialog.vue";
import { useHorizontalDrag } from "@/composables/useHorizontalDrag";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  apply: [];
  "open-dialog": [name: "bg" | "padding"];
}>();

const prefs = useReaderPrefsStore();
const theme = useThemeStore();

const weightItems = resources.arrays.text_font_weight as string[];
const indentOptions = resources.arrays.indent as string[];
const chineseModes = resources.arrays.chinese_mode as string[];
const fontVisible = ref(false);
const tipVisible = ref(false);
const selectorOpen = ref(false);
const selectorKind = ref<"weight" | "indent" | "chinese">("weight");

const selectorTitle = computed(() => {
  if (selectorKind.value === "weight") return "文章字重切换";
  if (selectorKind.value === "indent") return "缩进";
  return "中文简繁体转换";
});
const selectorItems = computed(() => {
  if (selectorKind.value === "weight") return weightItems;
  if (selectorKind.value === "indent") return indentOptions;
  return chineseModes;
});
const selectorSelected = computed(() => {
  if (selectorKind.value === "weight") return prefs.textBold;
  if (selectorKind.value === "indent") return prefs.paragraphIndent;
  return prefs.chineseMode;
});
const fontProgress = computed(() => Math.min(45, Math.max(0, prefs.fontSize - 5)));
const letterProgress = computed(() =>
  Math.min(100, Math.max(0, Math.round(prefs.letterSpacing * 100 + 50))),
);
const lineProgress = computed(() =>
  Math.min(20, Math.max(0, Math.round(prefs.lineSpacing * 10))),
);
const paragraphProgress = computed(() =>
  Math.min(20, Math.max(0, Math.round(prefs.paragraphSpacing))),
);

function update(patch: Parameters<typeof prefs.patch>[0]) {
  prefs.patch(patch);
  emit("apply");
}

function openSelector(kind: "weight" | "indent" | "chinese") {
  selectorKind.value = kind;
  selectorOpen.value = true;
}

async function onSelectorSelect(index: number) {
  if (selectorKind.value === "weight") {
    update({ textBold: index });
    return;
  }
  if (selectorKind.value === "indent") {
    update({ paragraphIndent: index });
    return;
  }
  update({ chineseMode: index });
  try {
    await kvSet("chineseConverterType", String(index));
  } catch {
    // ignore; reader falls back to original text
  }
}

function setFontProgress(value: number) {
  update({ fontSize: value + 5 });
}

function setLetterProgress(value: number) {
  update({ letterSpacing: (value - 50) / 100 });
}

function setLineProgress(value: number) {
  update({ lineSpacing: value / 10 });
}

function setParagraphProgress(value: number) {
  update({ paragraphSpacing: value });
}

function updateAnim(index: number) {
  update({ pageAnim: index });
}

function selectScheme(index: number) {
  theme.setScheme(index);
  emit("apply");
}

let schemePressTimer: number | undefined;
let schemePressStart = { x: 0, y: 0 };
let schemeLongPressed = false;

function clearSchemePress() {
  if (schemePressTimer) {
    window.clearTimeout(schemePressTimer);
    schemePressTimer = undefined;
  }
}

function startSchemePress(event: PointerEvent, index: number) {
  if (event.button !== 0) return;
  clearSchemePress();
  schemePressStart = { x: event.clientX, y: event.clientY };
  schemeLongPressed = false;
  schemePressTimer = window.setTimeout(() => {
    schemePressTimer = undefined;
    schemeLongPressed = true;
    editScheme(index);
  }, 500);
}

function moveSchemePress(event: PointerEvent) {
  if (!schemePressTimer) return;
  if (
    Math.abs(event.clientX - schemePressStart.x) > 4 ||
    Math.abs(event.clientY - schemePressStart.y) > 4
  ) {
    clearSchemePress();
  }
}

function endSchemePress() {
  clearSchemePress();
}

const {
  dragging: styleDragging,
  wasDragged: styleWasDragged,
  onPointerDown: onStyleListPointerDown,
  onPointerMove: onStyleListPointerMove,
  onPointerUp: onStyleListPointerUp,
  onPointerCancel: onStyleListPointerCancel,
  onClickCapture: onStyleListClickCapture,
} = useHorizontalDrag(() => {
  clearSchemePress();
  schemeLongPressed = false;
});

function onSchemeClick(index: number) {
  if (schemeLongPressed || styleWasDragged()) {
    schemeLongPressed = false;
    return;
  }
  selectScheme(index);
}

function onStyleWheel(event: WheelEvent) {
  const element = event.currentTarget as HTMLElement | null;
  if (!element) return;
  element.scrollLeft += event.deltaY || event.deltaX;
}

function editScheme(index: number) {
  theme.setScheme(index);
  emit("open-dialog", "bg");
}

function addScheme() {
  theme.addScheme("");
  emit("open-dialog", "bg");
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="sheet"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="style-panel" data-id="root_view">
      <!-- 顶部操作按钮行（dialog_read_book_style.xml） -->
      <div class="action-row">
        <button
          class="action-btn"
          data-id="text_font_weight_converter"
          @click="openSelector('weight')"
        >
          <span :class="{ accent: prefs.textBold === 0 }">中</span>/<span :class="{ accent: prefs.textBold === 1 }">粗</span>/<span :class="{ accent: prefs.textBold === 2 }">细</span>
        </button>
        <button class="action-btn" data-id="tv_text_font" @click="fontVisible = true">字体</button>
        <button class="action-btn" data-id="tv_text_indent" @click="openSelector('indent')">缩进</button>
        <button class="action-btn" data-id="chinese_converter" @click="openSelector('chinese')">
          <span :class="{ accent: prefs.chineseMode === 1 }">简</span>/<span :class="{ accent: prefs.chineseMode === 2 }">繁</span>
        </button>
        <button class="action-btn" data-id="tv_padding" @click="emit('open-dialog', 'padding')">边距</button>
        <button class="action-btn" data-id="tv_tip" @click="tipVisible = true">信息</button>
      </div>

      <!-- DetailSeekBar x4 -->
      <DetailSeekBar
        data-id="dsb_text_size"
        title="字号"
        :model-value="fontProgress"
        :max="45"
        :display="(value) => String(value + 5)"
        @update:model-value="setFontProgress"
      />
      <DetailSeekBar
        data-id="dsb_text_letter_spacing"
        title="字距"
        :model-value="letterProgress"
        :max="100"
        :display="(value) => ((value - 50) / 100).toFixed(2)"
        @update:model-value="setLetterProgress"
      />
      <DetailSeekBar
        data-id="dsb_line_size"
        title="行距"
        :model-value="lineProgress"
        :max="20"
        :display="(value) => ((value - 10) / 10).toFixed(1)"
        @update:model-value="setLineProgress"
      />
      <DetailSeekBar
        data-id="dsb_paragraph_spacing"
        title="段距"
        :model-value="paragraphProgress"
        :max="20"
        :display="(value) => (value / 10).toFixed(1)"
        @update:model-value="setParagraphProgress"
      />

      <p class="section-label" data-id="tv_page_anim">翻页动画</p>
      <div class="anim-group" data-id="rg_page_anim">
        <button data-id="rb_anim0" :class="{ active: prefs.pageAnim === 0 }" @click="updateAnim(0)">覆盖</button>
        <button data-id="rb_anim1" :class="{ active: prefs.pageAnim === 1 }" @click="updateAnim(1)">平移</button>
        <button data-id="rb_simulation_anim" :class="{ active: prefs.pageAnim === 2 }" @click="updateAnim(2)">仿真</button>
        <button data-id="rb_scroll_anim" :class="{ active: prefs.pageAnim === 3 }" @click="updateAnim(3)">滚动</button>
        <button data-id="rb_no_anim" :class="{ active: prefs.pageAnim === 4 }" @click="updateAnim(4)">无动画</button>
      </div>

      <div class="bg-header">
        <p class="section-label" data-id="tv_bg_ts">文字颜色和背景颜色（自定义）</p>
        <label class="share-layout" data-id="tv_share_layout">
          共用布局
          <input
            type="checkbox"
            data-id="cb_share_layout"
            :checked="prefs.shareLayout"
            @change="update({ shareLayout: ($event.target as HTMLInputElement).checked })"
          />
        </label>
      </div>

      <!-- 阅读背景横向列表（item_read_style） -->
      <div
        class="style-list"
        :class="{ dragging: styleDragging }"
        data-id="rv_style"
        @wheel.prevent="onStyleWheel"
        @pointerdown="onStyleListPointerDown"
        @pointermove="onStyleListPointerMove"
        @pointerup="onStyleListPointerUp"
        @pointercancel="onStyleListPointerCancel"
        @click.capture="onStyleListClickCapture"
      >
        <button
          v-for="(scheme, index) in theme.schemes"
          :key="index"
          class="style-item"
          :class="{ active: theme.currentScheme === index }"
          :title="scheme.name"
          @pointerdown="startSchemePress($event, index)"
          @pointermove="moveSchemePress"
          @pointerup="endSchemePress"
          @pointerleave="endSchemePress"
          @pointercancel="endSchemePress"
          @click="onSchemeClick(index)"
          @contextmenu.prevent="editScheme(index)"
          @dragstart.prevent
        >
          <span
            class="swatch"
            data-id="iv_style"
            :style="{
              background: scheme.day.bgValue,
              color: scheme.day.textColor,
              borderColor:
                theme.currentScheme === index
                  ? 'var(--ac-accent)'
                  : scheme.day.textColor,
            }"
          >
            {{ scheme.name.slice(0, 1) || "文" }}
          </span>
          <span class="scheme-name">{{ scheme.name }}</span>
        </button>
        <button class="style-item" title="添加" @click="addScheme">
          <span class="swatch add" data-id="iv_style">＋</span>
        </button>
      </div>
    </div>

    <FontSelectDialog v-model="fontVisible" @apply="emit('apply')" />
    <TipConfigDialog v-model="tipVisible" @apply="emit('apply')" />
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
.style-panel {
  display: flex;
  flex-direction: column;
}
.action-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  padding: 16px 16px 0;
}
.action-btn {
  flex: none;
  border: 1px solid var(--ac-primary-text);
  border-radius: 3px;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 14px;
  line-height: 1.25;
  padding: 4px 6px;
  cursor: pointer;
  white-space: nowrap;
}
.action-btn:hover {
  background: var(--ac-transparent-30);
}
.action-btn .accent {
  color: var(--ac-accent);
}
.section-label {
  margin: 10px 16px 4px;
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.anim-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 0 16px 6px;
}
.anim-group button {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 7px 12px;
  font-size: 13px;
  cursor: pointer;
}
.anim-group button.active {
  border-color: var(--ac-primary);
  color: var(--ac-primary);
}
.bg-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-right: 16px;
}
.bg-header .section-label {
  margin-right: 0;
}
.share-layout {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--ac-secondary-text);
  white-space: nowrap;
}
.style-list {
  display: flex;
  flex-wrap: nowrap;
  gap: 12px;
  width: 100%;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 8px 16px 16px;
  scrollbar-width: none;
  touch-action: pan-y;
  overscroll-behavior-x: contain;
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
  -webkit-overflow-scrolling: touch;
}
.style-list.dragging {
  cursor: grabbing;
}
.style-list::-webkit-scrollbar {
  display: none;
}
.style-item {
  flex: 0 0 auto;
  border: none;
  background: transparent;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 0;
  cursor: pointer;
  max-width: 64px;
  -webkit-touch-callout: none;
}
.swatch {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: 2px solid var(--ac-divider);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  box-sizing: border-box;
}
.swatch.add {
  border-style: dashed;
  background: transparent;
  color: var(--ac-secondary-text);
  font-size: 20px;
}
.scheme-name {
  max-width: 64px;
  font-size: 11px;
  color: var(--ac-secondary-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
