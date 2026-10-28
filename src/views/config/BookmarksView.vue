<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { Bookmark } from "@/api/bindings";
import { bookmarkDelete, bookmarkList } from "@/api/bookmarks";
import TitleBar from "@/components/android/TitleBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();
const bookmarks = ref<Bookmark[]>([]);
const message = ref("");

async function load() {
  bookmarks.value = await bookmarkList();
}

async function remove(bookmark: Bookmark) {
  if (!confirm("删除该书签？")) return;
  await bookmarkDelete(bookmark.time);
  await load();
  message.value = "书签已删除";
}

function open(bookmark: Bookmark) {
  if (!bookmark.bookUrl) return;
  router.push({
    path: "/reader",
    query: { bookUrl: bookmark.bookUrl, chapter: String(bookmark.chapterIndex ?? 0) },
  });
}

onMounted(load);
</script>

<template>
  <div class="bookmarks-page">
    <TitleBar title="全部书签" @back="router.back()" />
    <p v-if="message" class="message">{{ message }}</p>
    <main class="list">
      <button
        v-for="bookmark in bookmarks"
        :key="bookmark.time"
        class="bookmark"
        @click="open(bookmark)"
        @contextmenu.prevent="remove(bookmark)"
      >
        <span class="chapter">{{ bookmark.chapterName }}</span>
        <span class="book-text">{{ bookmark.bookText }}</span>
        <span v-if="bookmark.content" class="content">{{ bookmark.content }}</span>
      </button>
      <EmptyState v-if="bookmarks.length === 0" icon="ic_bookmark" text="暂无书签" />
    </main>
  </div>
</template>

<style scoped>
.bookmarks-page {
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
.bookmark {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 0;
  text-align: left;
  cursor: pointer;
}
.bookmark:hover {
  background: var(--ac-transparent-10);
}
.chapter,
.book-text,
.content {
  padding: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chapter {
  font-size: 14px;
}
.book-text,
.content {
  font-size: 12px;
  color: var(--ac-secondary-text);
}
</style>
