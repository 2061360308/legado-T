<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import { bookList } from "@/core/application/books";
import { readRecordClear, readRecordDelete, readRecordList } from "@/core/application/readRecords";
import type { Book, ReadRecord } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import SearchBar from "@/components/android/SearchBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const prefs = useReaderPrefsStore();
const records = ref<ReadRecord[]>([]);
const books = ref<Book[]>([]);
const keyword = ref("");
const sort = ref<"name" | "time" | "last">("name");
const message = ref("");
const menuVisible = ref(false);

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "sortName", label: "按名称", checked: sort.value === "name" },
  { key: "sortTime", label: "按阅读时长", checked: sort.value === "time" },
  { key: "sortLast", label: "按最后阅读", checked: sort.value === "last" },
  { key: "enableRecord", label: "启用阅读记录", checked: prefs.enableReadRecord },
]);

const bookByName = computed(() => {
  const map = new Map<string, Book>();
  for (const book of books.value) map.set(book.name, book);
  return map;
});

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  const list = records.value.filter(
    (item) =>
      !key ||
      item.bookName.toLowerCase().includes(key) ||
      (item.author || "").toLowerCase().includes(key),
  );
  return [...list].sort((a, b) => {
    if (sort.value === "time") return b.readTime - a.readTime;
    if (sort.value === "last") return b.lastRead - a.lastRead;
    return a.bookName.localeCompare(b.bookName, "zh-CN");
  });
});

const totalSeconds = computed(() =>
  records.value.reduce((sum, item) => sum + Math.max(0, item.readTime), 0),
);

function fmt(ts?: number) {
  return ts ? new Date(ts * 1000).toLocaleString("zh-CN") : "";
}

function duration(seconds: number) {
  if (seconds < 60) return `${seconds} 秒`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)} 分钟`;
  return `${(seconds / 3600).toFixed(1)} 小时`;
}

function open(item: ReadRecord) {
  const book = bookByName.value.get(item.bookName);
  if (!book) {
    message.value = "原书籍已不在书架";
    return;
  }
  router.push({ path: "/reader", query: { bookUrl: book.bookUrl } });
}

async function load() {
  records.value = await readRecordList();
  books.value = await bookList();
}

async function remove(item: ReadRecord) {
  await readRecordDelete(item.deviceId, item.bookName);
  await load();
}

async function clearAll() {
  if (!confirm("清空全部阅读记录？")) return;
  await readRecordClear();
  await load();
  message.value = "阅读记录已清空";
}

function onMenu(key: string) {
  if (key === "sortName") sort.value = "name";
  else if (key === "sortTime") sort.value = "time";
  else if (key === "sortLast") sort.value = "last";
  else if (key === "enableRecord") {
    prefs.patch({ enableReadRecord: !prefs.enableReadRecord });
  }
}

onMounted(load);
</script>

<template>
  <div class="records-page">
    <TitleBar search-only @back="router.back()">
      <template #content>
        <SearchBar v-model="keyword" placeholder="搜索书名、作者" />
      </template>
      <template #actions>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>
    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />

    <div class="summary-row">
      <span class="summary-label">全部阅读时间</span>
      <span class="summary-value">{{ duration(totalSeconds) }}</span>
      <button @click="clearAll">清空</button>
    </div>

    <p v-if="message" class="message">{{ message }}</p>
    <main ref="listRef" class="list">
      <div v-for="item in filtered" :key="`${item.deviceId}-${item.bookName}`" class="record-row">
        <button class="record" @click="open(item)">
          <span class="name">{{ item.bookName }}</span>
          <span class="line">
            <span class="tag">阅读时长</span>
            <span class="value">{{ duration(item.readTime) }}</span>
          </span>
          <span class="line">
            <span class="tag">最后阅读</span>
            <span class="value">{{ fmt(item.lastRead) }}</span>
          </span>
        </button>
        <button class="clear" @click="remove(item)">清空</button>
      </div>
      <p v-if="filtered.length === 0" class="empty">暂无阅读记录</p>
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
.summary-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--ac-divider);
}
.summary-label,
.summary-value {
  padding: 6px;
  font-size: 13px;
}
.summary-label {
  color: var(--ac-primary-text);
}
.summary-value {
  flex: 1;
  color: var(--ac-secondary-text);
}
.summary-row button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 13px;
  padding: 6px;
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
.record-row {
  display: flex;
  align-items: center;
  padding: 0 10px;
}
.record {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 4px 0;
  text-align: left;
}
.record:active {
  background: var(--ac-transparent-10);
}
.name {
  width: 100%;
  padding: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
}
.line {
  display: flex;
  align-items: center;
  max-width: 100%;
}
.tag,
.value {
  padding: 6px;
  font-size: 12px;
  color: var(--ac-secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.clear {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 13px;
  padding: 10px;
}
.empty {
  margin: 40px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
