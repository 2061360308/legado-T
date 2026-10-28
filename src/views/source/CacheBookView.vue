<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { bookList } from "@/api/books";
import type { Book } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();
const books = ref<Book[]>([]);
const message = ref("");

function clearCache(book: Book) {
  message.value = `《${book.name}》缓存清理暂未接入`;
}

onMounted(async () => {
  books.value = await bookList();
});
</script>

<template>
  <div class="cache-page">
    <TitleBar title="离线缓存" @back="router.back()" />
    <p v-if="message" class="message">{{ message }}</p>
    <main class="list">
      <div v-for="book in books" :key="book.bookUrl" class="row">
        <span class="name">{{ book.name }}</span>
        <button @click="clearCache(book)">清除缓存</button>
      </div>
      <EmptyState v-if="books.length === 0" icon="ic_import" text="暂无可缓存书籍" />
    </main>
  </div>
</template>

<style scoped>
.cache-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  overflow-y: auto;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
}
button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 13px;
  padding: 6px 8px;
  cursor: pointer;
}
</style>
