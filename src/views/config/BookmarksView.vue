<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { Bookmark } from "@/core/domain";
import { bookmarkDelete, bookmarkList } from "@/core/application/bookmarks";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import BookmarkDialog from "@/views/reader/dialogs/BookmarkDialog.vue";

const router = useRouter();
const bookmarks = ref<Bookmark[]>([]);
const message = ref("");
const editing = ref<Bookmark | null>(null);
const dialogVisible = ref(false);
const menuVisible = ref(false);

const menuItems: PopupMenuItem[] = [
  { key: "exportJson", label: "导出 JSON" },
  { key: "exportMd", label: "导出 Markdown" },
];

const grouped = computed(() => {
  const map = new Map<string, { title: string; items: Bookmark[] }>();
  for (const item of bookmarks.value) {
    const title = `${item.bookName || "未知书籍"}（${item.bookAuthor || "未知作者"}）`;
    if (!map.has(title)) map.set(title, { title, items: [] });
    map.get(title)!.items.push(item);
  }
  return [...map.values()];
});

async function load() {
  bookmarks.value = await bookmarkList();
}

async function remove(bookmark: Bookmark) {
  if (!confirm("删除这条书签？")) return;
  await bookmarkDelete(bookmark.time);
  await load();
  message.value = "书签已删除";
}

function open(bookmark: Bookmark) {
  if (!bookmark.bookUrl) return;
  router.push({
    path: "/reader",
    query: {
      bookUrl: bookmark.bookUrl,
      chapter: String(bookmark.chapterIndex ?? 0),
      chapterPos: String(bookmark.chapterPos ?? 0),
    },
  });
}

function edit(bookmark: Bookmark) {
  editing.value = bookmark;
  dialogVisible.value = true;
}

function exportBookmarks(kind: "json" | "md") {
  const content =
    kind === "json"
      ? JSON.stringify(bookmarks.value, null, 2)
      : bookmarks.value
          .map(
            (item) =>
              `## ${item.bookName || "未知书籍"} · ${item.chapterName || ""}\n\n${item.bookText || ""}\n\n${
                item.content ? `备注：${item.content}\n\n` : ""
              }`,
          )
          .join("");
  const blob = new Blob([content], {
    type: kind === "json" ? "application/json" : "text/markdown",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `bookmarks.${kind === "json" ? "json" : "md"}`;
  anchor.click();
  URL.revokeObjectURL(url);
  message.value = `已导出 ${bookmarks.value.length} 条书签`;
}

function onMenu(key: string) {
  if (key === "exportJson") exportBookmarks("json");
  else if (key === "exportMd") exportBookmarks("md");
}

onMounted(load);
</script>

<template>
  <div class="bookmarks-page">
    <TitleBar title="全部书签" @back="router.back()">
      <template #actions>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>
    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />
    <p v-if="message" class="message">{{ message }}</p>
    <main class="list">
      <section v-for="group in grouped" :key="group.title" class="bookmark-group">
        <h2 class="group-title">{{ group.title }}</h2>
        <div v-for="bookmark in group.items" :key="bookmark.time" class="bookmark-row">
          <button class="bookmark" @click="open(bookmark)">
            <span class="chapter">{{ bookmark.chapterName }}</span>
            <span class="book-text">{{ bookmark.bookText }}</span>
            <span v-if="bookmark.content" class="content">{{ bookmark.content }}</span>
          </button>
          <button class="edit" @click="edit(bookmark)">编辑</button>
          <button class="delete" @click="remove(bookmark)">删除</button>
        </div>
      </section>
      <p v-if="bookmarks.length === 0" class="empty">暂无书签</p>
    </main>
    <BookmarkDialog v-model="dialogVisible" :bookmark="editing" @saved="load" />
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
.group-title {
  margin: 0;
  padding: 8px 16px;
  background: var(--ac-background-card);
  color: var(--ac-primary-text);
  font-size: 13px;
  font-weight: 500;
}
.bookmark-row {
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--ac-divider);
}
.bookmark {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 10px 16px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  text-align: left;
  cursor: pointer;
}
.bookmark:active {
  background: var(--ac-transparent-10);
}
.chapter,
.book-text,
.content {
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
.edit {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 12px;
  padding: 10px 4px;
}
.delete {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-error);
  font-size: 12px;
  padding: 10px;
}
.empty {
  margin: 40px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
