<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { bookFileSize, bookList } from "@/api/books";
import type { Book } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const router = useRouter();
const files = ref<{ book: Book; size: number }[]>([]);
const error = ref("");

function formatSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  return `${(size / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

onMounted(async () => {
  try {
    const books = await bookList();
    const rows = await Promise.all(
      books.map(async (book) => ({ book, size: await bookFileSize(book.bookUrl) })),
    );
    files.value = rows.sort((a, b) => a.book.name.localeCompare(b.book.name, "zh-CN"));
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div class="files-page">
    <TitleBar title="文件管理" @back="router.back()" />
    <main class="content">
      <p class="path">应用数据目录 / books</p>
      <p v-if="error" class="error">{{ error }}</p>
      <div class="file-list">
        <div v-for="file in files" :key="file.book.bookUrl" class="file-row">
          <Icon name="ic_import" :size="22" />
          <span>{{ file.book.name }}</span>
          <span class="size">{{ formatSize(file.size) }}</span>
        </div>
        <p v-if="files.length === 0 && !error" class="empty">目录内暂无书籍文件</p>
      </div>
      <button class="import" @click="router.push('/import')">导入书籍</button>
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
.file-row > span:first-of-type {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.file-row .size {
  flex: none;
  color: var(--ac-secondary-text);
  font-size: 12px;
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
