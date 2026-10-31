<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import { platform } from "@/runtime";
import { useAppearanceStore } from "@/stores/appearance";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const router = useRouter();
const appearance = useAppearanceStore();
const message = ref("");
const imageActionsVisible = ref(false);
const imagePart = ref<"day" | "night">("day");

const dayImageName = computed(() => fileName(appearance.welcomeImage));
const nightImageName = computed(() => fileName(appearance.welcomeImageDark));
const dayImageReady = computed(
  () => appearance.customWelcome && Boolean(appearance.welcomeImage),
);
const nightImageReady = computed(
  () => appearance.customWelcome && Boolean(appearance.welcomeImageDark),
);

function fileName(path: string | null): string {
  if (!path) return "未设置";
  const value = path.replace(/[\\/]+$/, "");
  const index = Math.max(value.lastIndexOf("\\"), value.lastIndexOf("/"));
  return index >= 0 ? value.slice(index + 1) : value;
}

function notify(text: string) {
  message.value = text;
}

async function chooseImage(part: "day" | "night") {
  try {
    const picked = await platform.pickThemeImage();
    if (picked.cancelled) return;
    appearance.setWelcomeImage(part, picked.path);
    notify(`已选择欢迎图片：${picked.name}`);
  } catch (error) {
    notify(error instanceof Error ? error.message : "选择欢迎图片失败");
  }
}

function openImage(part: "day" | "night") {
  const path = part === "day" ? appearance.welcomeImage : appearance.welcomeImageDark;
  if (!path) {
    void chooseImage(part);
    return;
  }
  imagePart.value = part;
  imageActionsVisible.value = true;
}

function reselectImage() {
  imageActionsVisible.value = false;
  void chooseImage(imagePart.value);
}

function removeImage() {
  appearance.setWelcomeImage(imagePart.value, null);
  appearance.setWelcomeShowText(imagePart.value, true);
  appearance.setWelcomeShowIcon(imagePart.value, true);
  imageActionsVisible.value = false;
  notify("已删除欢迎图片");
}
</script>

<template>
  <div class="welcome-config">
    <TitleBar title="欢迎页样式" @back="router.back()" />
    <main class="pref-scroll">
      <section class="pref-group">
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">自定义欢迎页</span>
            <span class="pref-summary">使用自定义启动界面</span>
          </span>
          <input
            type="checkbox"
            :checked="appearance.customWelcome"
            @change="appearance.setCustomWelcome(($event.target as HTMLInputElement).checked)"
          />
        </label>
      </section>

      <section class="pref-group">
        <h2>日间</h2>
        <button class="pref-row" @click="openImage('day')">
          <span class="pref-text">
            <span class="pref-label">背景图片</span>
            <span class="pref-summary">{{ dayImageName }}</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <label class="pref-row" :class="{ disabled: !dayImageReady }">
          <span class="pref-text">
            <span class="pref-label">显示文字</span>
            <span class="pref-summary">显示“阅读｜享受美好时光”</span>
          </span>
          <input
            type="checkbox"
            :disabled="!dayImageReady"
            :checked="appearance.welcomeShowText"
            @change="appearance.setWelcomeShowText('day', ($event.target as HTMLInputElement).checked)"
          />
        </label>
        <label class="pref-row" :class="{ disabled: !dayImageReady }">
          <span class="pref-text">
            <span class="pref-label">显示图标</span>
            <span class="pref-summary">显示默认书籍图标</span>
          </span>
          <input
            type="checkbox"
            :disabled="!dayImageReady"
            :checked="appearance.welcomeShowIcon"
            @change="appearance.setWelcomeShowIcon('day', ($event.target as HTMLInputElement).checked)"
          />
        </label>
      </section>

      <section class="pref-group">
        <h2>夜间</h2>
        <button class="pref-row" @click="openImage('night')">
          <span class="pref-text">
            <span class="pref-label">背景图片</span>
            <span class="pref-summary">{{ nightImageName }}</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <label class="pref-row" :class="{ disabled: !nightImageReady }">
          <span class="pref-text">
            <span class="pref-label">显示文字</span>
            <span class="pref-summary">显示“阅读｜享受美好时光”</span>
          </span>
          <input
            type="checkbox"
            :disabled="!nightImageReady"
            :checked="appearance.welcomeShowTextDark"
            @change="appearance.setWelcomeShowText('night', ($event.target as HTMLInputElement).checked)"
          />
        </label>
        <label class="pref-row" :class="{ disabled: !nightImageReady }">
          <span class="pref-text">
            <span class="pref-label">显示图标</span>
            <span class="pref-summary">显示默认书籍图标</span>
          </span>
          <input
            type="checkbox"
            :disabled="!nightImageReady"
            :checked="appearance.welcomeShowIconDark"
            @change="appearance.setWelcomeShowIcon('night', ($event.target as HTMLInputElement).checked)"
          />
        </label>
      </section>

      <p v-if="message" class="message">{{ message }}</p>
    </main>

    <AndroidDialog v-model="imageActionsVisible" mode="center" title="欢迎图片">
      <button class="action-row" @click="reselectImage">选择图片</button>
      <button class="action-row danger" @click="removeImage">删除</button>
    </AndroidDialog>
  </div>
</template>

<style scoped>
.welcome-config {
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
.pref-row.disabled {
  opacity: 0.6;
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
.message {
  margin: 8px 20px 0;
  color: var(--ac-primary);
  font-size: 12px;
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
