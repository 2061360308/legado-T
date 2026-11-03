<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import {
  bookSourceAddToShelf,
  bookSourceExplore,
} from "@/core/application/bookSources";
import { bookGet, bookList } from "@/core/application/books";
import type { OnlineSearchResult } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import CoverImage from "@/components/android/CoverImage.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import { showError } from "@/utils/errors";

const route = useRoute();
const router = useRouter();
const resultListRef = ref<HTMLElement | null>(null);
useScrollMemory(resultListRef);
const sourceUrl = computed(() => String(route.query.sourceUrl || ""));
const exploreUrl = computed(() => String(route.query.exploreUrl || ""));
const title = computed(
  () =>
    String(route.query.categoryTitle || "") ||
    String(route.query.sourceName || "发现"),
);
const results = ref<OnlineSearchResult[]>([]);
const loading = ref(false);
const message = ref("");
const page = ref(0);
const hasMore = ref(true);
const adding = ref("");
const shelfKeys = ref(new Set<string>());

function shelfKey(name: string, author: string): string {
  return `${name || ""}\u0000${author || ""}`;
}

async function loadShelf() {
  try {
    const books = await bookList();
    shelfKeys.value = new Set(
      books.flatMap((book) => [
        shelfKey(book.name, book.author),
        `url:${book.bookUrl}`,
      ]),
    );
  } catch {
    shelfKeys.value = new Set();
  }
}

function inShelf(item: OnlineSearchResult): boolean {
  if (item.bookUrl && shelfKeys.value.has(`url:${item.bookUrl}`)) return true;
  return shelfKeys.value.has(shelfKey(String(item.name || ""), String(item.author || "")));
}

async function loadPage(next: number, append = false) {
  if (!sourceUrl.value || !exploreUrl.value || loading.value) return;
  loading.value = true;
  message.value = "";
  try {
    const list = await bookSourceExplore(sourceUrl.value, exploreUrl.value, next);
    page.value = next;
    if (append) {
      const existing = new Set(results.value.map((item) => item.bookUrl || item.name));
      const fresh = list.filter((item) => {
        const key = item.bookUrl || item.name;
        if (existing.has(key)) return false;
        existing.add(key);
        return true;
      });
      results.value = [...results.value, ...fresh];
      hasMore.value = fresh.length > 0;
    } else {
      results.value = list;
      hasMore.value = list.length > 0;
    }
    if (!list.length) hasMore.value = false;
  } catch (error) {
    showError("加载发现页书籍", error);
    hasMore.value = false;
  } finally {
    loading.value = false;
  }
}

async function loadMore(force = false) {
  if (loading.value) return;
  if (!force && !hasMore.value) return;
  await loadPage(page.value + 1, true);
}

function onScroll(event: Event) {
  const element = event.currentTarget as HTMLElement;
  if (element.scrollHeight - element.scrollTop - element.clientHeight < 160) {
    void loadMore();
  }
}

function readPath(book: unknown): string {
  const type = Number((book as { type?: number } | null)?.type ?? 0);
  if (type & 2) return "/audio";
  if (type & 4) return "/manga";
  if (type & 8) return "/file-source";
  return "/reader";
}

async function openBook(item: OnlineSearchResult) {
  if (!item.bookUrl || adding.value) return;
  if (inShelf(item)) {
    try {
      const book = await bookGet(item.bookUrl);
      router.push({ path: readPath(book), query: { bookUrl: item.bookUrl } });
    } catch (error) {
      showError("打开书籍", error);
    }
    return;
  }
  adding.value = item.bookUrl;
  try {
    const book = await bookSourceAddToShelf(sourceUrl.value, item.bookUrl);
    await loadShelf();
    router.push({ path: readPath(book), query: { bookUrl: book.bookUrl } });
  } catch (error) {
    showError("加入书架并阅读", error);
  } finally {
    adding.value = "";
  }
}

onMounted(async () => {
  await loadShelf();
  await loadPage(1);
});
</script>

<template>
  <div class="explore-show">
    <TitleBar :title="title" @back="router.push('/explore')" />

    <p v-if="message" class="message">{{ message }}</p>
    <main ref="resultListRef" class="result-list" @scroll.passive="onScroll">
      <EmptyState
        v-if="!results.length && !loading"
        icon="ic_bottom_explore_e"
        text="该分类暂无书籍"
      />
      <article
        v-for="(item, index) in results"
        :key="`${item.bookUrl}-${index}`"
        class="book"
        @click="openBook(item)"
      >
        <div class="cover-wrap">
          <CoverImage
            :name="String(item.name || '')"
            :author="String(item.author || '')"
            :src="item.coverUrl ? String(item.coverUrl) : null"
            :width="80"
            :height="110"
          />
          <span v-if="inShelf(item)" class="shelf-dot" />
        </div>
        <div class="book-main">
          <p class="book-name">{{ item.name || "未命名" }}</p>
          <p class="book-meta">{{ item.author || "未知作者" }}</p>
          <p v-if="item.kind" class="book-kind">{{ item.kind }}</p>
          <p v-if="item.latestChapterTitle" class="book-meta">
            最新：{{ item.latestChapterTitle }}
          </p>
          <p v-if="item.intro" class="book-intro">{{ item.intro }}</p>
        </div>
      </article>

      <div v-if="results.length" class="load-more" @click="loadMore(true)">
        <span v-if="loading">加载中…</span>
        <span v-else-if="!hasMore">没有更多了</span>
        <span v-else>加载更多</span>
      </div>
      <div v-else-if="loading" class="load-more">加载中…</div>
    </main>
  </div>
</template>

<style scoped>
.explore-show {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.message {
  margin: 8px 16px;
  color: var(--ac-error);
  font-size: 12px;
}
.result-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.book {
  display: flex;
  gap: 10px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--ac-divider);
  cursor: pointer;
}
.book:active {
  background: var(--ac-transparent-10);
}
.cover-wrap {
  position: relative;
  flex: none;
}
.shelf-dot {
  position: absolute;
  left: 5px;
  top: 5px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #4caf50;
}
.book-main {
  flex: 1;
  min-width: 0;
}
.book-name {
  margin: 0 0 3px;
  color: var(--ac-primary-text);
  font-size: 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.book-meta {
  margin: 0 0 2px;
  color: var(--ac-secondary-text);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.book-kind {
  margin: 0 0 2px;
  color: var(--ac-primary);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.book-intro {
  margin: 4px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.load-more {
  padding: 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 12px;
  cursor: pointer;
}
</style>
