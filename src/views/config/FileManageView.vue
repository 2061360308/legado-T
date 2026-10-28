<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { appDataDir } from "@tauri-apps/api/path";
import { readDir } from "@tauri-apps/plugin-fs";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const router = useRouter();
const dataDir = ref("");
const files = ref<{ name: string; isFile: boolean }[]>([]);
const error = ref("");

onMounted(async () => {
  try {
    const dir = await appDataDir();
    dataDir.value = dir;
    const entries = await readDir(`${dir.replace(/[\\/]+$/, "")}/booksData`);
    files.value = entries
      .filter((entry) => entry.isFile)
      .map((entry) => ({ name: entry.name, isFile: true }))
      .sort((a, b) => a.name.localeCompare(b.name));
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div class="files-page">
    <TitleBar title="文件管理" @back="router.back()" />
    <main class="content">
      <p class="path">{{ dataDir }}/booksData</p>
      <p v-if="error" class="error">{{ error }}</p>
      <div class="file-list">
        <div v-for="file in files" :key="file.name" class="file-row">
          <Icon name="ic_import" :size="22" />
          <span>{{ file.name }}</span>
        </div>
        <p v-if="files.length === 0 && !error" class="empty">目录内暂无书籍文件</p>
      </div>
      <button class="import" @click="router.push('/import')">导入 TXT</button>
    </main>
  </div>
</template>

<style scoped>
.files-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 20px 24px;
}
.path {
  font-size: 12px;
  color: var(--ac-secondary-text);
  word-break: break-all;
}
.error {
  color: var(--ac-error);
  font-size: 12px;
}
.file-list {
  display: flex;
  flex-direction: column;
  margin-top: 10px;
}
.file-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 0;
  border-bottom: 1px solid var(--ac-divider);
  font-size: 13px;
  word-break: break-all;
}
.empty {
  text-align: center;
  color: var(--ac-secondary-text);
  padding: 30px 0;
  font-size: 13px;
}
.import {
  margin-top: 16px;
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 8px 16px;
  cursor: pointer;
}
</style>
