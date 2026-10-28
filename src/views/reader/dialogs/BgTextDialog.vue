<script setup lang="ts">
import { computed, ref } from "vue";
import {
  PRESET_SCHEMES,
  ThemeMode,
  useThemeStore,
  type ModeColors,
  type ReadStyleScheme,
} from "@/stores/theme";
import AndroidDialog from "./AndroidDialog.vue";
import SelectorDialog from "./SelectorDialog.vue";
import Icon from "@/components/android/Icon.vue";
import { useHorizontalDrag } from "@/composables/useHorizontalDrag";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const theme = useThemeStore();
const scheme = computed(() => theme.currentSchemeData);
const part = computed<"day" | "night" | "eink">(() => {
  if (theme.effectiveMode === ThemeMode.Night) return "night";
  if (theme.effectiveMode === ThemeMode.Eink) return "eink";
  return "day";
});
const modeColors = computed(() => scheme.value[part.value]);
const alphaPercent = computed(() =>
  Math.min(100, Math.max(0, Math.round((modeColors.value.bgAlpha / 255) * 100))),
);

const nameVisible = ref(false);
const nameDraft = ref("");
const restoreVisible = ref(false);
const message = ref("");
const importInput = ref<HTMLInputElement | null>(null);
const presetNames = PRESET_SCHEMES.map((item) => item.name);

const {
  dragging: bgDragging,
  onPointerDown: onBgListPointerDown,
  onPointerMove: onBgListPointerMove,
  onPointerUp: onBgListPointerUp,
  onPointerCancel: onBgListPointerCancel,
  onClickCapture: onBgListClickCapture,
} = useHorizontalDrag();

function cloneScheme(): ReadStyleScheme {
  return JSON.parse(JSON.stringify(scheme.value)) as ReadStyleScheme;
}

function updateMode(patch: Partial<ModeColors>) {
  const next = cloneScheme();
  next[part.value] = { ...next[part.value], ...patch };
  theme.updateScheme(theme.currentScheme, next);
}

function openName() {
  nameDraft.value = scheme.value.name;
  nameVisible.value = true;
}

function confirmName() {
  const next = cloneScheme();
  next.name = nameDraft.value.trim() || next.name;
  theme.updateScheme(theme.currentScheme, next);
  nameVisible.value = false;
}

function applyPreset(index: number) {
  theme.updateScheme(
    theme.currentScheme,
    JSON.parse(JSON.stringify(PRESET_SCHEMES[index])) as ReadStyleScheme,
  );
}

function removeScheme() {
  if (!theme.removeScheme(theme.currentScheme)) {
    message.value = "数量已是最少，不能删除";
    return;
  }
  emit("update:modelValue", false);
}

function exportScheme() {
  const blob = new Blob([JSON.stringify(scheme.value, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `reader-scheme-${scheme.value.name || "custom"}.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}

function importScheme(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    try {
      const parsed = JSON.parse(String(reader.result)) as ReadStyleScheme;
      if (!parsed.day || !parsed.night || !parsed.eink) return;
      theme.updateScheme(theme.currentScheme, {
        name: parsed.name || scheme.value.name,
        day: { ...scheme.value.day, ...parsed.day },
        night: { ...scheme.value.night, ...parsed.night },
        eink: { ...scheme.value.eink, ...parsed.eink },
      });
    } catch {
      message.value = "导入失败";
    }
  };
  reader.readAsText(file);
}

async function fileToDataUrl(file: File): Promise<string> {
  const dataUrl = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("read failed"));
    reader.readAsDataURL(file);
  });
  const image = await new Promise<HTMLImageElement>((resolve, reject) => {
    const element = new Image();
    element.onload = () => resolve(element);
    element.onerror = () => reject(new Error("image failed"));
    element.src = dataUrl;
  });
  const max = 1600;
  if (image.naturalWidth <= max && image.naturalHeight <= max) return dataUrl;
  const scale = Math.min(max / image.naturalWidth, max / image.naturalHeight);
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
  canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
  const ctx = canvas.getContext("2d");
  ctx?.drawImage(image, 0, 0, canvas.width, canvas.height);
  return canvas.toDataURL("image/jpeg", 0.85);
}

async function onPickImage(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  try {
    const dataUrl = await fileToDataUrl(file);
    updateMode({ bgImage: dataUrl, bgKind: "external" });
    message.value = "";
  } catch {
    message.value = "图片读取失败";
  }
}

function clearBgImage() {
  updateMode({ bgImage: null, bgKind: "color" });
}

function setAlpha(percent: number) {
  updateMode({ bgAlpha: Math.round((percent / 100) * 255) });
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="sheet"
    :show-header="false"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="bg-config">
      <div class="name-row">
        <span class="name-title">样式名称</span>
        <span class="name-value">{{ scheme.name || "文字" }}</span>
        <button class="icon-btn" aria-label="编辑名称" @click="openName">
          <Icon name="ic_edit" :size="20" />
        </button>
        <span class="spacer" />
        <button class="text-btn" @click="restoreVisible = true">还原</button>
      </div>

      <label class="switch-row">
        <span>深色状态栏图标</span>
        <input
          type="checkbox"
          :checked="modeColors.darkStatusIcon"
          @change="updateMode({ darkStatusIcon: ($event.target as HTMLInputElement).checked })"
        />
      </label>
      <label class="switch-row">
        <span>下划线</span>
        <input
          type="checkbox"
          :checked="modeColors.underline"
          @change="updateMode({ underline: ($event.target as HTMLInputElement).checked })"
        />
      </label>

      <div class="tools-row">
        <label class="chip" :style="{ borderColor: modeColors.textColor }">
          <span class="dot" :style="{ background: modeColors.textColor }" />
          文字颜色
          <input
            type="color"
            :value="modeColors.textColor"
            @input="updateMode({ textColor: ($event.target as HTMLInputElement).value })"
          />
        </label>
        <label class="chip" :style="{ borderColor: modeColors.bgValue }">
          <span class="dot" :style="{ background: modeColors.bgValue }" />
          背景颜色
          <input
            type="color"
            :value="modeColors.bgValue"
            @input="updateMode({ bgValue: ($event.target as HTMLInputElement).value })"
          />
        </label>
        <button class="icon-btn" title="导入" aria-label="导入" @click="importInput?.click()">
          <Icon name="ic_import" :size="22" />
        </button>
        <button class="icon-btn" title="导出" aria-label="导出" @click="exportScheme">
          <Icon name="ic_export" :size="22" />
        </button>
        <button class="icon-btn" title="删除" aria-label="删除" @click="removeScheme">
          <Icon name="ic_clear_all" :size="22" />
        </button>
        <input
          ref="importInput"
          class="file-input"
          type="file"
          accept="application/json,.json"
          @change="importScheme"
        />
      </div>

      <p class="section-label">背景透明度</p>
      <div class="alpha-row">
        <input
          type="range"
          min="0"
          max="100"
          :value="alphaPercent"
          @input="setAlpha(Number(($event.target as HTMLInputElement).value))"
        />
        <span class="alpha-value">{{ alphaPercent }}%</span>
      </div>

      <p class="section-label">背景图片</p>
      <div
        class="bg-list"
        :class="{ dragging: bgDragging }"
        @pointerdown="onBgListPointerDown"
        @pointermove="onBgListPointerMove"
        @pointerup="onBgListPointerUp"
        @pointercancel="onBgListPointerCancel"
        @click.capture="onBgListClickCapture"
      >
        <button
          class="bg-item none"
          :class="{ active: !modeColors.bgImage }"
          @click="clearBgImage"
        >
          无
        </button>
        <div v-if="modeColors.bgImage" class="bg-item active">
          <img :src="modeColors.bgImage" alt="" />
        </div>
        <label class="bg-item add" title="选择图片">
          <Icon name="ic_image" :size="22" />
          <span>选择图片</span>
          <input
            class="file-input"
            type="file"
            accept="image/*"
            @change="onPickImage"
          />
        </label>
      </div>

      <p v-if="message" class="message">{{ message }}</p>
    </div>

    <AndroidDialog
      v-model="nameVisible"
      mode="center"
      title="样式名称"
      width="min(92vw, 420px)"
    >
      <input v-model="nameDraft" class="name-input" placeholder="name" />
      <template #footer>
        <button class="footer-btn" @click="nameVisible = false">取消</button>
        <button class="footer-btn primary" @click="confirmName">确定</button>
      </template>
    </AndroidDialog>

    <SelectorDialog
      v-model="restoreVisible"
      title="选择预设布局"
      :items="presetNames"
      @select="applyPreset"
    />
  </AndroidDialog>
</template>

<style scoped>
.bg-config {
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
}
.name-title {
  font-size: 16px;
  color: var(--ac-primary-text);
}
.name-value {
  font-size: 14px;
  color: var(--ac-secondary-text);
  padding: 0 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 45%;
}
.spacer {
  flex: 1;
}
.text-btn {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 14px;
  padding: 4px 8px;
  cursor: pointer;
}
.icon-btn {
  width: 32px;
  height: 32px;
  flex: none;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--ac-primary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
}
.icon-btn:hover {
  background: var(--ac-transparent-10);
}
.switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 6px;
  font-size: 14px;
  color: var(--ac-primary-text);
}
.switch-row input {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.tools-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
}
.chip {
  flex: 1;
  min-width: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px solid var(--ac-primary-text);
  border-radius: 3px;
  padding: 6px;
  font-size: 14px;
  color: var(--ac-primary-text);
  cursor: pointer;
  position: relative;
  white-space: nowrap;
}
.chip input[type="color"] {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}
.dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid var(--ac-divider);
}
.section-label {
  margin: 8px 6px 2px;
  font-size: 13px;
  color: var(--ac-primary-text);
}
.alpha-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 2px 6px 6px;
}
.alpha-row input {
  flex: 1;
  min-width: 0;
  accent-color: var(--ac-primary);
}
.alpha-value {
  flex: none;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.bg-list {
  display: flex;
  gap: 8px;
  width: 100%;
  min-width: 0;
  overflow-x: auto;
  padding: 6px;
  scrollbar-width: thin;
  touch-action: pan-y;
  overscroll-behavior-x: contain;
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
}
.bg-list.dragging {
  cursor: grabbing;
}
.bg-item {
  flex: 0 0 auto;
  width: 62px;
  height: 62px;
  border: 1px solid var(--ac-divider);
  border-radius: 3px;
  background: transparent;
  color: var(--ac-secondary-text);
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  font-size: 11px;
  cursor: pointer;
  overflow: hidden;
  padding: 0;
}
.bg-item.active {
  border-color: var(--ac-accent);
}
.bg-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  -webkit-user-drag: none;
}
.bg-item.add span {
  font-size: 10px;
}
.file-input {
  display: none;
}
.message {
  margin: 0;
  padding: 4px 6px;
  font-size: 12px;
  color: var(--ac-error);
}
.name-input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
}
.footer-btn {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 14px;
  padding: 8px 12px;
  cursor: pointer;
}
</style>
