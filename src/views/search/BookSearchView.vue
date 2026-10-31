<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { bookList } from "@/core/application/books";
import type { Book } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const HISTORY_KEY = "local-reader:search-history";
const PRECISION_KEY = "local-reader:search-precision";

const router = useRouter();
const keyword = ref("");
const menuVisible = ref(false);
const message = ref("");
const books = ref<Book[]>([]);
const history = ref<string[]>(loadHistory());
const precision = ref(localStorage.getItem(PRECISION_KEY) === "1");

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "precision", label: "精准搜索", checked: precision.value },
  { key: "source", label: "书源管理", icon: "ic_cfg_source" },
  { key: "log", label: "日志", icon: "ic_cfg_about" },
]);

const results = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  if (!key) return [];
  return books.value.filter((book) => {
    const name = book.name.toLowerCase();
    const author = (book.author || "").toLowerCase();
    if (name.includes(key) || author.includes(key)) return true;
    if (precision.value) return false;
    return (book.customIntro || book.intro || "").toLowerCase().includes(key);
  });
});

function loadHistory(): string[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter((item) => typeof item === "string") : [];
  } catch {
    return [];
  }
}

function saveHistory() {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value));
}

function onMenu(key: string) {
  if (key === "precision") {
    precision.value = !precision.value;
    localStorage.setItem(PRECISION_KEY, precision.value ? "1" : "0");
  } else if (key === "source") {
    router.push("/book-source");
  } else if (key === "log") {
    message.value = "日志暂未接入";
  }
}

function search() {
  const key = keyword.value.trim();
  if (!key) return;
  history.value = [key, ...history.value.filter((item) => item !== key)].slice(0, 20);
  saveHistory();
  message.value = results.value.length === 0 ? "书架中没有匹配的书籍" : "";
}

function chooseHistory(item: string) {
  keyword.value = item;
  search();
}

function removeHistory(item: string) {
  history.value = history.value.filter((value) => value !== item);
  saveHistory();
}

function clearHistory() {
  if (!history.value.length) return;
  if (!confirm("清空全部搜索历史？")) return;
  history.value = [];
  saveHistory();
}

function openBook(book: Book) {
  router.push({ path: "/book-info", query: { bookUrl: book.bookUrl } });
}

function kindTags(book: Book): string[] {
  return (book.kind || "")
    .split(/[,;，；]/)
    .map((value) => value.trim())
    .filter(Boolean)
    .slice(0, 4);
}

function intro(book: Book): string {
  return (book.customIntro || book.intro || "").replace(/\s+/g, " ").trim();
}

onMounted(async () => {
  try {
    books.value = await bookList();
  } catch (e) {
    message.value = String(e);
  }
});
</script>

<template>
  <div class="book-search">
    <TitleBar title="搜索" @back="router.back()">
      <template #content>
        <label class="search-box">
          <Icon name="ic_search_hint" :size="18" />
          <input
            v-model="keyword"
            placeholder="搜索书名、作者"
            @keydown.enter="search"
          />
        </label>
      </template>
      <template #actions>
        <button aria-label="搜索" @click="search"><Icon name="ic_search" :size="24" /></button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <div class="progress" />
    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />

    <main class="content">
      <p v-if="message" class="message">{{ message }}</p>

      <template v-if="keyword.trim()">
        <div v-if="results.length" class="result-list" data-id="recycler_view">
          <button
            v-for="book in results"
            :key="book.bookUrl"
            class="result-card"
            @click="openBook(book)"
          >
            <CoverImage
              :name="book.name"
              :author="book.author"
              :src="book.customCoverUrl || book.coverUrl"
              :width="80"
              :height="110"
            />
            <span class="result-main">
              <span class="name-row">
                <span class="name">{{ book.name }}</span>
                <span class="shelf-dot" title="已在书架" />
              </span>
              <span class="author">作者：{{ book.author || "未知作者" }}</span>
              <span v-if="kindTags(book).length" class="kind-row">
                <span v-for="tag in kindTags(book)" :key="tag" class="kind-tag">{{ tag }}</span>
              </span>
              <span v-if="book.latestChapterTitle" class="latest">
                最新：{{ book.latestChapterTitle }}
              </span>
              <span v-if="intro(book)" class="intro">{{ intro(book) }}</span>
            </span>
          </button>
        </div>
        <div v-else class="empty-result">
          <Icon name="ic_bottom_books_e" :size="56" />
          <span>书架中没有匹配的书籍</span>
        </div>
      </template>

      <template v-else>
        <div class="history-header">
          <span class="section-title">搜索历史</span>
          <button v-if="history.length" @click="clearHistory">清空</button>
        </div>
        <div v-if="history.length" class="history-chips">
          <button
            v-for="item in history"
            :key="item"
            class="history-chip"
            @click="chooseHistory(item)"
            @contextmenu.prevent="removeHistory(item)"
          >
            {{ item }}
          </button>
        </div>
        <p v-else class="empty">暂无搜索历史</p>
      </template>
    </main>
  </div>
</template>

<style scoped>
.book-search {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.search-box {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  height: 30px;
  margin: 10px 0;
  padding: 0 10px;
  border-radius: 15px;
  background: rgba(255, 255, 255, 0.16);
  color: inherit;
}
.search-box input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: inherit;
  font-size: 13px;
}
.search-box input::placeholder {
  color: inherit;
  opacity: 0.75;
}
.progress {
  height: 2px;
  background: transparent;
}
.content {
  flex: 1;
  overflow-y: auto;
  position: relative;
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.result-list {
  display: flex;
  flex-direction: column;
}
.result-card {
  width: 100%;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  gap: 8px;
  padding: 8px;
  text-align: left;
  cursor: pointer;
}
.result-card:active {
  background: var(--ac-transparent-10);
}
.result-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 2px 0;
}
.name-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}
.name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 16px;
  font-weight: 500;
}
.shelf-dot {
  flex: none;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--md-green-600, #43a047);
}
.author,
.latest,
.intro {
  font-size: 12px;
  color: var(--ac-secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.intro {
  white-space: normal;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  line-height: 1.4;
}
.kind-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.kind-tag {
  max-width: 100%;
  padding: 1px 6px;
  border-radius: 2px;
  background: var(--ac-primary);
  color: #fff;
  font-size: 11px;
}
.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px 6px 6px;
}
.section-title {
  padding: 6px;
  font-size: 14px;
  color: var(--ac-primary-text);
}
.history-header button {
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  font-size: 13px;
  padding: 6px;
}
.history-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 4px 12px;
}
.history-chip {
  max-width: 100%;
  border: 1px solid var(--ac-divider);
  border-radius: 4px;
  background: var(--ac-background-menu);
  color: var(--ac-primary-text);
  padding: 6px 12px;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.empty {
  margin: 8px 12px;
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.empty-result {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 80px 16px;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
