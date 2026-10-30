<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import { pickThemeImage } from "@/platform";
import { useAppearanceStore } from "@/stores/appearance";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const router = useRouter();
const appearance = useAppearanceStore();
const message = ref("");
const coverActionsVisible = ref(false);
const coverPart = ref<"day" | "night">("day");

const dayCoverName = computed(() => fileName(appearance.defaultCover));
const nightCoverName = computed(() => fileName(appearance.defaultCoverDark));

function fileName(path: string | null): string {
  if (!path) return "未设置";
  const value = path.replace(/[\\/]+$/, "");
  const index = Math.max(value.lastIndexOf("\\"), value.lastIndexOf("/"));
  return index >= 0 ? value.slice(index + 1) : value;
}

function notify(text: string) {
  message.value = text;
}

async function chooseCover(part: "day" | "night") {
  try {
    const picked = await pickThemeImage();
    if (picked.cancelled) return;
    appearance.setDefaultCover(part, picked.path);
    notify(`已选择默认封面：${picked.name}`);
  } catch (error) {
    notify(error instanceof Error ? error.message : "选择默认封面失败");
  }
}

function openCover(part: "day" | "night") {
  const path = part === "day" ? appearance.defaultCover : appearance.defaultCoverDark;
  if (!path) {
    void chooseCover(part);
    return;
  }
  coverPart.value = part;
  coverActionsVisible.value = true;
}

function reselectCover() {
  coverActionsVisible.value = false;
  void chooseCover(coverPart.value);
}

function removeCover() {
  appearance.setDefaultCover(coverPart.value, null);
  coverActionsVisible.value = false;
  notify("已删除默认封面");
}
</script>

<template>
  <div class="cover-page">
    <TitleBar title="封面配置" @back="router.back()" />
    <main class="pref-scroll">
      <section class="pref-group">
        <h2>日间</h2>
        <button class="pref-row" @click="openCover('day')">
          <span class="pref-text">
            <span class="pref-label">默认封面</span>
            <span class="pref-summary">{{ dayCoverName }}</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">显示书名</span>
            <span class="pref-summary">在默认封面显示书名</span>
          </span>
          <input
            type="checkbox"
            :checked="appearance.coverShowName"
            @change="appearance.setCoverShowName('day', ($event.target as HTMLInputElement).checked)"
          />
        </label>
        <label class="pref-row" :class="{ disabled: !appearance.coverShowName }">
          <span class="pref-text">
            <span class="pref-label">显示作者</span>
            <span class="pref-summary">在默认封面显示作者</span>
          </span>
          <input
            type="checkbox"
            :disabled="!appearance.coverShowName"
            :checked="appearance.coverShowAuthor"
            @change="appearance.setCoverShowAuthor('day', ($event.target as HTMLInputElement).checked)"
          />
        </label>
      </section>

      <section class="pref-group">
        <h2>夜间</h2>
        <button class="pref-row" @click="openCover('night')">
          <span class="pref-text">
            <span class="pref-label">默认封面</span>
            <span class="pref-summary">{{ nightCoverName }}</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">显示书名</span>
            <span class="pref-summary">在默认封面显示书名</span>
          </span>
          <input
            type="checkbox"
            :checked="appearance.coverShowNameDark"
            @change="appearance.setCoverShowName('night', ($event.target as HTMLInputElement).checked)"
          />
        </label>
        <label class="pref-row" :class="{ disabled: !appearance.coverShowNameDark }">
          <span class="pref-text">
            <span class="pref-label">显示作者</span>
            <span class="pref-summary">在默认封面显示作者</span>
          </span>
          <input
            type="checkbox"
            :disabled="!appearance.coverShowNameDark"
            :checked="appearance.coverShowAuthorDark"
            @change="appearance.setCoverShowAuthor('night', ($event.target as HTMLInputElement).checked)"
          />
        </label>
      </section>

      <p v-if="message" class="message">{{ message }}</p>
    </main>

    <AndroidDialog v-model="coverActionsVisible" mode="center" title="默认封面">
      <button class="action-row" @click="reselectCover">选择图片</button>
      <button class="action-row danger" @click="removeCover">删除</button>
    </AndroidDialog>
  </div>
</template>

<style scoped>
.cover-page {
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
