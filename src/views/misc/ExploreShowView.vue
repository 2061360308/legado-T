<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  bookSourceAddToShelf,
  bookSourceExplore,
  bookSourceExploreCategories,
} from "@/core/application/bookSources";
import type { ExploreCategory, OnlineSearchResult } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import CoverImage from "@/components/android/CoverImage.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.sourceUrl || ""));
const sourceName = computed(() => String(route.query.sourceName || "发现"));
const categories = ref<ExploreCategory[]>([]);
const active = ref<ExploreCategory | null>(null);
const results = ref<OnlineSearchResult[]>([]);
const loading = ref(false);
const message = ref("");
const page = ref(1);
const adding = ref("");

async function loadCategories() {
  if (!sourceUrl.value) return;
  try {
    categories.value = await bookSourceExploreCategories(sourceUrl.value);
    const first = categories.value.find((item) => String(item.url || "").trim());
    if (first) {
      active.value = first;
      await loadPage(1);
    }
  } catch (error) {
    message.value = String(error);
  }
}

async function loadPage(next: number) {
  if (!active.value) return;
  const url = String(active.value.url || "").trim();
  if (!url) return;
  loading.value = true;
  message.value = "";
  try {
    page.value = Math.max(1, next);
    results.value = await bookSourceExplore(sourceUrl.value, url, page.value);
  } catch (error) {
    message.value = String(error);
    results.value = [];
  } finally {
    loading.value = false;
  }
}

async function selectCategory(category: ExploreCategory) {
  if (!String(category.url || "").trim()) return;
  active.value = category;
  results.value = [];
  await loadPage(1);
}

async function addToShelf(item: OnlineSearchResult) {
  if (!item.bookUrl || adding.value) return;
  adding.value = item.bookUrl;
  try {
    const book = await bookSourceAddToShelf(sourceUrl.value, item.bookUrl);
    router.push({ path: "/book-info", query: { bookUrl: book.bookUrl } });
  } catch (error) {
    message.value = String(error);
  } finally {
    adding.value = "";
  }
}

onMounted(loadCategories);
</script>

<template>
  <div class="explore-show">
    <TitleBar :title="sourceName" @back="router.push('/explore')" />

    <div v-if="categories.length" class="categories">
      <button
        v-for="category in categories"
        :key="`${category.title}-${category.url}`"
        class="category"
        :class="{
          active: active?.title === category.title && active?.url === category.url,
          disabled: !String(category.url || '').trim(),
        }"
        @click="selectCategory(category)"
      >
        {{ category.title }}
      </button>
    </div>

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>

    <main v-else class="result-list">
      <EmptyState
        v-if="!results.length"
        icon="ic_bottom_explore_e"
        text="该分类暂无书籍"
      />
      <article v-for="(item, index) in results" :key="`${item.bookUrl}-${index}`" class="book">
        <CoverImage
          :name="String(item.name || '')"
          :author="String(item.author || '')"
          :src="item.coverUrl ? String(item.coverUrl) : null"
          :width="68"
          :height="94"
        />
        <div class="book-main">
          <p class="book-name">{{ item.name || "未命名" }}</p>
          <p class="book-meta">{{ item.author || "未知作者" }}<template v-if="item.kind"> · {{ item.kind }}</template></p>
          <p v-if="item.latestChapterTitle" class="book-meta">最新：{{ item.latestChapterTitle }}</p>
          <p v-if="item.intro" class="book-intro">{{ item.intro }}</p>
        </div>
        <button class="add" :disabled="!item.bookUrl || adding === item.bookUrl" @click="addToShelf(item)">
          加入书架
        </button>
      </article>

      <div v-if="results.length" class="pager">
        <button :disabled="page <= 1 || loading" @click="loadPage(page - 1)">上一页</button>
        <span>第 {{ page }} 页</span>
        <button :disabled="loading" @click="loadPage(page + 1)">下一页</button>
      </div>
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
.categories {
  display: flex;
  gap: 6px;
  padding: 6px 10px;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
}
.category {
  flex: none;
  border: none;
  background: var(--ac-transparent-10);
  color: var(--ac-primary-text);
  border-radius: 14px;
  padding: 5px 12px;
  font-size: 12px;
  white-space: nowrap;
}
.category.active {
  background: var(--ac-primary);
  color: #fff;
}
.category.disabled {
  opacity: 0.4;
}
.message,
.loading {
  margin: 8px 16px;
  color: var(--ac-secondary-text);
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
}
.book-main {
  flex: 1;
  min-width: 0;
}
.book-name {
  margin: 0 0 3px;
  color: var(--ac-primary-text);
  font-size: 15px;
}
.book-meta {
  margin: 0 0 2px;
  color: var(--ac-secondary-text);
  font-size: 11px;
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
.add {
  align-self: center;
  flex: none;
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  padding: 5px 8px;
  font-size: 12px;
}
.pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 12px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.pager button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 5px 10px;
}
</style>
