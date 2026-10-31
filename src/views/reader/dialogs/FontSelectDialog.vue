<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Icon from "@/components/android/Icon.vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import {
  SYSTEM_FONTS,
  customFontFamily,
  importCustomFont,
  loadActiveFont,
  loadCustomFonts,
  loadPreviewFont,
  systemFontFamily,
  type FontFile,
} from "@/utils/customFonts";
import AndroidDialog from "./AndroidDialog.vue";
import SelectorDialog from "./SelectorDialog.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  apply: [];
}>();

const prefs = useReaderPrefsStore();
const fonts = ref<FontFile[]>([]);
const previews = ref<Record<string, string>>({});
const loading = ref(false);
const error = ref("");
const systemVisible = ref(false);

const systemItems = SYSTEM_FONTS.map((font) => font.name);
const systemSelected = computed(() => (prefs.customFontPath ? -1 : prefs.systemTypeface));
const activeMissing = computed(
  () =>
    Boolean(prefs.customFontPath) &&
    !fonts.value.some((font) => font.path === prefs.customFontPath),
);

async function load() {
  loading.value = true;
  error.value = "";
  try {
    fonts.value = await loadCustomFonts();
    const loaded: Record<string, string> = {};
    await Promise.all(
      fonts.value.map(async (font) => {
        const family = await loadPreviewFont(font.path);
        if (family) loaded[font.path] = family;
      }),
    );
    previews.value = loaded;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) {
      systemVisible.value = false;
      return;
    }
    void load();
  },
);

async function chooseFont(font: FontFile) {
  error.value = "";
  try {
    if (!(await loadActiveFont(font.path))) throw new Error("字体加载失败");
    prefs.patch({
      customFontPath: font.path,
      customFontName: font.name,
      fontFamily: customFontFamily(prefs.systemTypeface),
    });
    emit("apply");
    emit("update:modelValue", false);
  } catch (e) {
    error.value = String(e);
  }
}

function chooseSystem(index: number) {
  const font = SYSTEM_FONTS[index] ?? SYSTEM_FONTS[0];
  prefs.patch({
    customFontPath: "",
    customFontName: "",
    systemTypeface: font.value,
    fontFamily: systemFontFamily(font.value),
  });
  systemVisible.value = false;
  emit("apply");
  emit("update:modelValue", false);
}

async function addFont() {
  error.value = "";
  try {
    const font = await importCustomFont();
    if (!font) return;
    await load();
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="选择字体"
    width="min(94vw, 760px)"
    height="86vh"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #actions>
      <button @click="systemVisible = true">默认</button>
      <button @click="addFont">其他</button>
    </template>

    <div class="font-panel">
      <p v-if="error" class="message error">{{ error }}</p>
      <p v-if="activeMissing" class="message warning">当前字体文件不可用，已回退系统字体。</p>
      <p v-if="loading" class="message">加载字体中…</p>
      <div v-else-if="fonts.length" class="font-list">
        <button
          v-for="font in fonts"
          :key="font.path"
          class="font-row"
          :class="{ active: prefs.customFontPath === font.path }"
          @click="chooseFont(font)"
        >
          <span
            class="font-name"
            :style="{ fontFamily: previews[font.path] || undefined }"
          >
            {{ font.name }}
          </span>
          <Icon
            v-if="prefs.customFontPath === font.path"
            name="ic_check"
            :size="22"
            class="check"
          />
        </button>
      </div>
      <p v-else class="message empty">暂无字体文件，点击右上角“其他”导入 TTF/OTF。</p>
    </div>

    <SelectorDialog
      v-model="systemVisible"
      title="默认字体"
      :items="systemItems"
      :selected="systemSelected"
      @select="chooseSystem"
    />
  </AndroidDialog>
</template>

<style scoped>
.font-panel {
  min-height: 100%;
  padding: 8px 0 12px;
}
.message {
  margin: 8px 20px;
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.message.error {
  color: var(--ac-error);
}
.message.warning {
  color: var(--ac-primary);
}
.message.empty {
  margin-top: 24px;
  text-align: center;
}
.font-list {
  display: flex;
  flex-direction: column;
}
.font-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 54px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px 20px;
  text-align: left;
  cursor: pointer;
}
.font-row.active {
  color: var(--ac-primary);
}
.font-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 17px;
}
.check {
  flex: none;
  color: var(--ac-primary);
}
</style>
