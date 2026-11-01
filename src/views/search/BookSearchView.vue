<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import {
  bookSourceAddToShelf,
  bookSourceSearch,
  bookSourceSearchHistory,
} from "@/core/application/bookSources";
import type { OnlineSearchResult } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const HISTORY_KEY = "local-reader:search-history";
const router = useRouter();
const keyword = ref("");
const menuVisible = ref(false);
const message = ref("");
const searching = ref(false);
const results = ref<OnlineSearchResult[]>([]);
const history = ref<string[]>(loadHistory());

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "source", label: "书源管理", icon: "ic_cfg_source" },
  { key: "log", label: "日志", icon: "ic_cfg_about" },
]);

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
  if (key === "source") router.push("/book-source");
  else if (key === "log") message.value = "日志暂未接入";
}

async function search() {
  const key = keyword.value.trim();
  if (!key || searching.value) return;
  history.value = [key, ...history.value.filter((item) => item !== key)].slice(0, 20);
  saveHistory();
  searching.value = true;
  message.value = "";
  try {
    const found = await bookSourceSearch(key);
    results.value = found;
    const failures = found.filter((item) => item.error);
    if (!found.length) message.value = "没有找到结果";
    else if (failures.length) message.value = `${failures.length} 个书源失败，已显示其他结果`;
  } catch (error) {
    message.value = String(error);
  } finally {
    searching.value = false;
  }
}

function chooseHistory(item: string) {
  keyword.value = item;
  void search();
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

async function addToShelf(result: OnlineSearchResult) {
  if (result.error || !result.bookUrl) {
    message.value = result.error || "书籍地址为空";
    return;
  }
  const sourceUrl = String(result.bookSourceUrl || result.origin || "");
  if (!sourceUrl) {
    message.value = "书源地址为空";
    return;
  }
  try {
    const book = await bookSourceAddToShelf(sourceUrl, result.bookUrl);
    router.push({ path: "/book-info", query: { bookUrl: book.bookUrl } });
  } catch (error) {
    message.value = String(error);
  }
}

onMounted(async () => {
  try {
    const saved = await bookSourceSearchHistory(20);
    if (saved.length) {
      history.value = saved;
      saveHistory();
    }
  } catch {
    // keep local history
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
        <button aria-label="搜索" :disabled="searching" @click="search">
          <Icon name="ic_search" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />

    <main class="content">
      <p v-if="message" class="message">{{ message }}</p>
      <p v-if="searching" class="loading">正在搜索…</p>

      <template v-if="keyword.trim()">
        <div v-if="results.length" class="result-list">
          <div
            v-for="(result, index) in results"
            :key="`${result.bookSourceUrl}-${result.bookUrl}-${index}`"
            class="result-card"
          >
            <CoverImage
              :name="String(result.name || '')"
              :author="String(result.author || '')"
              :src="result.coverUrl ? String(result.coverUrl) : null"
              :width="80"
              :height="110"
            />
            <span class="result-main">
              <span class="name">{{ result.error ? "书源失败" : result.name }}</span>
              <span class="author">
                {{ result.error ? result.bookSourceName : `作者：${result.author || "未知作者"}` }}
              </span>
              <span v-if="result.bookSourceName" class="source">
                来源：{{ result.bookSourceName }}
              </span>
              <span v-if="result.error" class="error">{{ result.error }}</span>
              <span v-else-if="result.intro" class="intro">{{ result.intro }}</span>
            </span>
            <button
              v-if="!result.error"
              class="add"
              :disabled="!result.bookUrl"
              @click="addToShelf(result)"
            >
              加入书架
            </button>
          </div>
        </div>
        <p v-else-if="!searching" class="empty-result">没有找到结果</p>
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
.loading {
  margin: 6px 16px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.result-list {
  display: flex;
  flex-direction: column;
}
.result-card {
  display: flex;
  gap: 8px;
  padding: 8px;
  border-bottom: 1px solid var(--ac-divider);
}
.result-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name {
  font-size: 14px;
  font-weight: 500;
}
.author,
.source,
.intro,
.error {
  color: var(--ac-secondary-text);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.error {
  color: var(--ac-error);
}
.add {
  align-self: center;
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  padding: 5px 8px;
  font-size: 12px;
}
.empty-result,
.empty {
  margin: 16px;
  color: var(--ac-secondary-text);
  text-align: center;
}
.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 6px;
}
.section-title {
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.history-header button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
}
.history-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 0 16px;
}
.history-chip {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 14px;
  padding: 5px 10px;
}
</style>
