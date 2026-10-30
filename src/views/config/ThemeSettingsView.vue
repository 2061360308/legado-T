<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import { useThemeStore } from "@/stores/theme";
import { useAppearanceStore, type LauncherIcon } from "@/stores/appearance";
import { getPlatformInfo, pickThemeImage } from "@/platform";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import NumberPickerDialog from "@/views/reader/dialogs/NumberPickerDialog.vue";
import defaultIcon from "@/assets/icons/default.png";
import launcher1Icon from "@/assets/icons/launcher1.svg";
import launcher2Icon from "@/assets/icons/launcher2.svg";
import launcher3Icon from "@/assets/icons/launcher3.svg";
import launcher4Icon from "@/assets/icons/launcher4.svg";
import launcher5Icon from "@/assets/icons/launcher5.svg";
import launcher6Icon from "@/assets/icons/launcher6.svg";

const router = useRouter();
const theme = useThemeStore();
const message = ref("");
const elevationVisible = ref(false);
const fontScaleVisible = ref(false);
const themeListVisible = ref(false);
const saveThemeVisible = ref(false);
const savePart = ref<"day" | "night">("day");
const themeName = ref("");
const bgActionsVisible = ref(false);
const bgBlurVisible = ref(false);
const bgPart = ref<"day" | "night">("day");
const bgBlurValue = ref(0);
const appearance = useAppearanceStore();
const iconDialogVisible = ref(false);
const supportsLauncherIcon = computed(() => {
  const info = getPlatformInfo();
  return info?.os === "android" && info.apiLevel >= 26;
});
const launcherOptions: { value: LauncherIcon; label: string; src: string }[] = [
  { value: "default", label: "默认图标", src: defaultIcon },
  { value: "launcher1", label: "原版图标 1", src: launcher1Icon },
  { value: "launcher2", label: "原版图标 2", src: launcher2Icon },
  { value: "launcher3", label: "原版图标 3", src: launcher3Icon },
  { value: "launcher4", label: "原版图标 4", src: launcher4Icon },
  { value: "launcher5", label: "原版图标 5", src: launcher5Icon },
  { value: "launcher6", label: "原版图标 6", src: launcher6Icon },
];
const currentIconSrc = computed(
  () => launcherOptions.find((item) => item.value === appearance.launcherIcon)?.src ?? defaultIcon,
);

const dayColors = [
  ["primary", "主色"],
  ["accent", "强调色"],
  ["background", "背景色"],
  ["bottomBackground", "导航栏色"],
] as const;
const nightColors = dayColors;

const dayBgName = computed(() => fileName(theme.dayBgImage));
const nightBgName = computed(() => fileName(theme.nightBgImage));

function fileName(path: string | null): string {
  if (!path) return "未设置";
  const value = path.replace(/[\\/]+$/, "");
  const index = Math.max(value.lastIndexOf("\\"), value.lastIndexOf("/"));
  return index >= 0 ? value.slice(index + 1) : value;
}

function update(
  part: "day" | "night",
  key: "primary" | "accent" | "background" | "bottomBackground",
  value: string,
) {
  theme.updateGlobal(part, key, value);
}

function notify(text: string) {
  message.value = text;
}

function chooseIcon(icon: LauncherIcon) {
  appearance.setLauncherIcon(icon);
  iconDialogVisible.value = false;
  notify(`已切换为${launcherOptions.find((item) => item.value === icon)?.label ?? "图标"}`);
}

function confirmElevation(value: number) {
  theme.setBarElevation(value);
}

function confirmFontScale(value: number) {
  theme.setFontScale(value);
}

function openSaveTheme(part: "day" | "night") {
  savePart.value = part;
  themeName.value = part === "day" ? "日间主题" : "夜间主题";
  saveThemeVisible.value = true;
}

function saveTheme() {
  const name = theme.saveGlobalTheme(savePart.value, themeName.value);
  saveThemeVisible.value = false;
  notify(`已保存${savePart.value === "day" ? "日间" : "夜间"}主题「${name}」`);
}

function applyTheme(index: number) {
  theme.applySavedTheme(index);
  themeListVisible.value = false;
  notify(`已应用主题「${theme.savedThemes[index]?.name ?? ""}」`);
}

function removeTheme(index: number) {
  const name = theme.savedThemes[index]?.name ?? "";
  if (!confirm(`删除主题「${name}」？`)) return;
  theme.removeSavedTheme(index);
  notify(`已删除主题「${name}」`);
}

async function chooseBackground(part: "day" | "night") {
  try {
    const picked = await pickThemeImage();
    if (picked.cancelled) return;
    theme.setBackgroundImage(part, picked.path);
    notify(`已选择背景图片：${picked.name}`);
  } catch (error) {
    notify(error instanceof Error ? error.message : "选择背景图片失败");
  }
}

function openBackground(part: "day" | "night") {
  const path = part === "day" ? theme.dayBgImage : theme.nightBgImage;
  if (!path) {
    void chooseBackground(part);
    return;
  }
  bgPart.value = part;
  bgActionsVisible.value = true;
}

function openBlur() {
  bgActionsVisible.value = false;
  bgBlurValue.value = bgPart.value === "day" ? theme.dayBgBlur : theme.nightBgBlur;
  bgBlurVisible.value = true;
}

function confirmBlur(value: number) {
  theme.setBackgroundBlur(bgPart.value, value);
  notify(`背景模糊度：${value}`);
}

function removeBackground() {
  theme.setBackgroundImage(bgPart.value, null);
  bgActionsVisible.value = false;
  notify("已删除背景图片");
}

function reselectBackground() {
  bgActionsVisible.value = false;
  void chooseBackground(bgPart.value);
}
</script>

<template>
  <div class="theme-page">
    <TitleBar title="主题设置" @back="router.back()" />
    <main class="pref-scroll">
      <section class="pref-group">
        <button v-if="supportsLauncherIcon" class="pref-row" @click="iconDialogVisible = true">
          <span class="pref-text">
            <span class="pref-label">更换图标</span>
            <span class="pref-summary">切换桌面启动图标样式</span>
          </span>
          <img class="icon-preview" :src="currentIconSrc" alt="" />
        </button>
        <button class="pref-row" @click="router.push('/welcome-config')">
          <span class="pref-text">
            <span class="pref-label">欢迎页样式</span>
            <span class="pref-summary">启动欢迎页图片与显示内容</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">沉浸状态栏</span>
            <span class="pref-summary">状态栏颜色透明</span>
          </span>
          <input
            type="checkbox"
            :checked="theme.transparentStatusBar"
            @change="theme.setTransparentStatusBar(($event.target as HTMLInputElement).checked)"
          />
        </label>
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">沉浸导航栏</span>
            <span class="pref-summary">导航栏颜色透明</span>
          </span>
          <input
            type="checkbox"
            :checked="theme.immNavigationBar"
            @change="theme.setImmNavigationBar(($event.target as HTMLInputElement).checked)"
          />
        </label>
        <button class="pref-row" @click="elevationVisible = true">
          <span class="pref-text">
            <span class="pref-label">标题栏阴影高度</span>
            <span class="pref-summary">当前阴影大小：{{ theme.barElevation }}dp</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="fontScaleVisible = true">
          <span class="pref-text">
            <span class="pref-label">字体缩放</span>
            <span class="pref-summary">字体缩放：{{ Math.round(theme.fontScale * 10) }}%</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="router.push('/cover-config')">
          <span class="pref-text">
            <span class="pref-label">封面配置</span>
            <span class="pref-summary">默认封面、显示书名与作者</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="themeListVisible = true">
          <span class="pref-text">
            <span class="pref-label">主题列表</span>
            <span class="pref-summary">保存、应用、删除主题</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
      </section>

      <section class="pref-group">
        <h2>日间</h2>
        <label v-for="[key, label] in dayColors" :key="key" class="pref-row">
          <span class="pref-text">
            <span class="pref-label">{{ label }}</span>
            <span class="pref-summary">{{ theme.globalTokens.day[key] }}</span>
          </span>
          <input
            type="color"
            :value="theme.globalTokens.day[key]"
            @input="update('day', key, ($event.target as HTMLInputElement).value)"
          />
        </label>
        <button class="pref-row" @click="openBackground('day')">
          <span class="pref-text">
            <span class="pref-label">背景图片</span>
            <span class="pref-summary">{{ dayBgName }}</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="openSaveTheme('day')">
          <span class="pref-text">
            <span class="pref-label">保存主题配置</span>
            <span class="pref-summary">保存当前日间配色</span>
          </span>
        </button>
      </section>

      <section class="pref-group">
        <h2>夜间</h2>
        <label v-for="[key, label] in nightColors" :key="key" class="pref-row">
          <span class="pref-text">
            <span class="pref-label">{{ label }}</span>
            <span class="pref-summary">{{ theme.globalTokens.night[key] }}</span>
          </span>
          <input
            type="color"
            :value="theme.globalTokens.night[key]"
            @input="update('night', key, ($event.target as HTMLInputElement).value)"
          />
        </label>
        <button class="pref-row" @click="openBackground('night')">
          <span class="pref-text">
            <span class="pref-label">背景图片</span>
            <span class="pref-summary">{{ nightBgName }}</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="openSaveTheme('night')">
          <span class="pref-text">
            <span class="pref-label">保存主题配置</span>
            <span class="pref-summary">保存当前夜间配色</span>
          </span>
        </button>
      </section>

      <section class="pref-group">
        <button class="pref-row" @click="router.push('/read-schemes')">
          <span class="pref-text">
            <span class="pref-label">阅读排版方案</span>
            <span class="pref-summary">阅读页背景、文字色与排版方案</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
      </section>

      <p v-if="message" class="message">{{ message }}</p>
    </main>

    <NumberPickerDialog
      v-model="elevationVisible"
      title="标题栏阴影高度"
      :min="0"
      :max="32"
      :value="theme.barElevation"
      @confirm="confirmElevation"
    />
    <NumberPickerDialog
      v-model="fontScaleVisible"
      title="字体缩放"
      :min="8"
      :max="16"
      :value="theme.fontScale"
      @confirm="confirmFontScale"
    />
    <NumberPickerDialog
      v-model="bgBlurVisible"
      title="背景图片模糊度"
      :min="0"
      :max="25"
      :value="bgBlurValue"
      @confirm="confirmBlur"
    />

    <AndroidDialog v-model="iconDialogVisible" mode="center" title="更换图标">
      <div class="icon-list">
        <button
          v-for="item in launcherOptions"
          :key="item.value"
          class="icon-option"
          :class="{ active: appearance.launcherIcon === item.value }"
          @click="chooseIcon(item.value)"
        >
          <img class="icon-option-preview" :src="item.src" alt="" />
          <span class="pref-text">
            <span class="pref-label">{{ item.label }}</span>
          </span>
          <span class="check">{{ appearance.launcherIcon === item.value ? "✓" : "" }}</span>
        </button>
      </div>
    </AndroidDialog>

    <AndroidDialog v-model="themeListVisible" mode="center" title="主题列表">
      <div v-if="theme.savedThemes.length === 0" class="empty">暂无已保存的主题</div>
      <div v-else class="theme-list">
        <div v-for="(item, index) in theme.savedThemes" :key="`${item.name}-${index}`" class="theme-item">
          <button class="theme-main" @click="applyTheme(index)">
            <span class="theme-swatch" :style="{ background: item.colors.background, color: item.colors.primary }">
              {{ item.name.slice(0, 1) }}
            </span>
            <span class="pref-text">
              <span class="pref-label">
                {{ item.name }}
                <span class="theme-tag">{{ item.isNightTheme ? "夜间" : "日间" }}</span>
              </span>
              <span class="pref-summary">
                {{ item.colors.primary }} / {{ item.colors.background }}
              </span>
            </span>
          </button>
          <button class="delete" aria-label="删除" @click="removeTheme(index)">
            <Icon name="ic_clear_all" :size="20" />
          </button>
        </div>
      </div>
    </AndroidDialog>

    <AndroidDialog v-model="saveThemeVisible" mode="center" title="主题名称">
      <input v-model="themeName" class="name-input" placeholder="name" />
      <template #footer>
        <button class="dialog-btn" @click="saveThemeVisible = false">取消</button>
        <button class="dialog-btn primary" @click="saveTheme">确定</button>
      </template>
    </AndroidDialog>

    <AndroidDialog v-model="bgActionsVisible" mode="center" title="背景图片">
      <button class="action-row" @click="openBlur">背景图片模糊度</button>
      <button class="action-row" @click="reselectBackground">选择图片</button>
      <button class="action-row danger" @click="removeBackground">删除</button>
    </AndroidDialog>
  </div>
</template>

<style scoped>
.theme-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.pref-scroll {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 24px;
}
.pref-group h2 {
  margin: 0;
  padding: 10px 20px 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--ac-primary);
}
.pref-row {
  width: 100%;
  min-height: 52px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  text-align: left;
  cursor: pointer;
}
.pref-row:hover {
  background: var(--ac-transparent-10);
}
.pref-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.pref-label {
  font-size: 15px;
}
.theme-tag {
  margin-left: 6px;
  padding: 1px 5px;
  border-radius: 2px;
  background: var(--ac-transparent-10);
  color: var(--ac-secondary-text);
  font-size: 11px;
  font-weight: 400;
}
.pref-summary {
  font-size: 12px;
  color: var(--ac-secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chevron {
  opacity: 0.5;
}
input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
input[type="color"] {
  width: 42px;
  height: 30px;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: transparent;
  padding: 1px;
}
.message {
  margin: 8px 20px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.empty {
  padding: 16px 0;
  color: var(--ac-secondary-text);
  font-size: 14px;
}
.theme-list,
.icon-list {
  display: flex;
  flex-direction: column;
}
.icon-option {
  width: 100%;
  min-height: 48px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0;
  text-align: left;
  cursor: pointer;
}
.icon-option.active {
  color: var(--ac-primary);
}
.icon-preview,
.icon-option-preview {
  width: 42px;
  height: 42px;
  flex: none;
  object-fit: contain;
  border-radius: 50%;
}
.icon-option-preview {
  width: 46px;
  height: 46px;
  margin: 6px 0;
  border: 1px solid var(--ac-divider);
}
.check {
  color: var(--ac-primary);
  font-size: 16px;
}
.theme-item {
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--ac-divider);
}
.theme-main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 10px 0;
  text-align: left;
  cursor: pointer;
}
.theme-swatch {
  width: 36px;
  height: 36px;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--ac-divider);
  font-size: 14px;
}
.delete {
  width: 40px;
  height: 40px;
  border: none;
  background: transparent;
  color: var(--ac-error);
  cursor: pointer;
}
.name-input {
  width: 100%;
  height: 42px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 0 10px;
  font-size: 15px;
}
.dialog-btn {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 14px;
  padding: 8px 12px;
  cursor: pointer;
}
.action-row {
  width: 100%;
  min-height: 48px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 15px;
  text-align: left;
  padding: 0;
  cursor: pointer;
}
.action-row.danger {
  color: var(--ac-error);
}
</style>
