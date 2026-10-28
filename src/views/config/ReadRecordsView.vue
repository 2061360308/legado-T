<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { bookList } from "@/api/books";
import type { Book } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const router = useRouter();
const books = ref<Book[]>([]);

const records = computed(() =>
  [...books.value]
    .filter((book) => book.durChapterTitle)
    .sort((a, b) => (b.durChapterTime ?? 0) - (a.durChapterTime ?? 0)),
);

function fmt(ts?: number) {
  return ts ? new Date(ts * 1000).toLocaleString("zh-CN") : "";
}

function open(book: Book) {
  router.push({ path: "/reader", query: { bookUrl: book.bookUrl } });
}

onMounted(async () => {
  books.value = await bookList();
});
</script>

<template>
  <div class="records-page">
    <TitleBar title="阅读记录" @back="router.back()" />
    <main class="list">
      <button v-for="book in records" :key="book.bookUrl" class="record" @click="open(book)">
        <CoverImage :name="book.name" :width="44" :height="60" />
        <span class="text">
          <span class="name">{{ book.name }}</span>
          <span class="chapter">
            第 {{ (book.durChapterIndex ?? 0) + 1 }} 章 · {{ book.durChapterTitle }}
          </span>
          <span class="time">{{ fmt(book.durChapterTime) }}</span>
        </span>
      </button>
      <p v-if="records.length === 0" class="empty">暂无阅读记录</p>
    </main>
  </div>
</template>

<style scoped>
.records-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.list {
  flex: 1;
  overflow-y: auto;
}
.record {
  width: 100%;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  gap: 12px;
  padding: 12px 20px;
  text-align: left;
  cursor: pointer;
}
.record:hover {
  background: var(--ac-transparent-10);
}
.text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.name {
  font-size: 15px;
  font-weight: 500;
}
.chapter {
  font-size: 12px;
  color: var(--ac-secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.time {
  font-size: 11px;
  color: var(--ac-secondary-text);
}
.empty {
  text-align: center;
  color: var(--ac-secondary-text);
  padding: 40px 0;
  font-size: 13px;
}
</style>
