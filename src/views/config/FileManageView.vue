<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { bookDelete, bookFileSize, bookList } from "@/core/application/books";
import type { Book } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const router = useRouter();
const files = ref<{ book: Book; size: number }[]>([]);
const error = ref("");
const message = ref("");
const total = ref(0);
const keyword = ref("");

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  if (!key) return files.value;
  return files.value.filter(
    (file) =>
      file.book.name.toLowerCase().includes(key) ||
      (file.book.author || "").toLowerCase().includes(key),
  );
});

function open(file: Book) {
  router.push({ path: "/book-info", query: { bookUrl: file.bookUrl } });
}

function formatSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  return `${(size / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

async function load() {
  error.value = "";
  try {
    const books = await bookList();
    const rows = await Promise.all(
      books.map(async (book) => ({ book, size: await bookFileSize(book.bookUrl) })),
    );
    files.value = rows.sort((a, b) => a.book.name.localeCompare(b.book.name, "zh-CN"));
    total.value = rows.reduce((sum, row) => sum + row.size, 0);
  } catch (e) {
    error.value = String(e);
  }
}

async function remove(book: Book) {
  if (!confirm(`删除《${book.name}》及其缓存文件？`)) return;
  try {
    await bookDelete(book.bookUrl, true);
    await load();
    message.value = `已删除《${book.name}》`;
  } catch (e) {
    message.value = String(e);
  }
}

onMounted(load);
</script>

<template>
  <div class="files-page">
    <TitleBar title="文件管理" @back="router.back()">
      <template #content>
        <input v-model="keyword" class="search" placeholder="搜索文件" />
      </template>
    </TitleBar>
    <main class="content">
      <p class="path">应用数据目录 / books</p>
      <p class="summary">共 {{ files.length }} 本，占用 {{ formatSize(total) }}</p>
      <p v-if="error" class="error">{{ error }}</p>
      <p v-if="message" class="message">{{ message }}</p>
      <div class="file-list">
        <div
          v-for="file in filtered"
          :key="file.book.bookUrl"
          class="file-row"
          @click="open(file.book)"
        >
          <Icon name="ic_import" :size="22" />
          <span class="name">{{ file.book.name }}</span>
          <span class="size">{{ formatSize(file.size) }}</span>
          <button @click.stop="remove(file.book)">删除</button>
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
.search {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
  height: 30px;
  border: none;
  border-radius: 15px;
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
  padding: 0 10px;
  outline: none;
}
.path,
.summary {
  font-size: 12px;
  color: var(--ac-secondary-text);
  word-break: break-all;
}
.summary {
  margin-top: 4px;
}
.error {
  color: var(--ac-error);
  font-size: 12px;
}
.message {
  color: var(--ac-primary);
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
  gap: 10px;
  padding: 11px 0;
  border-bottom: 1px solid var(--ac-divider);
  font-size: 13px;
  word-break: break-all;
}
.file-row .name {
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
.file-row button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-error);
  font-size: 12px;
  padding: 4px;
  cursor: pointer;
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
