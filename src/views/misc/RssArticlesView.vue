<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  rssArticleClear,
  rssArticles,
  rssReadRecordAdd,
  rssSortUrls,
  rssSourceList,
  rssSourceSave,
  type RssSort,
} from "@/core/application/rss";
import { platform } from "@/runtime";
import type { RssArticle, RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import RssReadRecordDialog from "./RssReadRecordDialog.vue";

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.sourceUrl || ""));
const sourceName = computed(() => String(route.query.sourceName || "订阅"));
const source = ref<RssSource | null>(null);
const articles = ref<RssArticle[]>([]);
const activeSort = ref(0);
const page = ref(1);
const next = ref<string | null>(null);
const loading = ref(false);
const refreshing = ref(false);
const message = ref("");
const menuVisible = ref(false);
const readRecordVisible = ref(false);
let pullStart = 0;
let pulling = false;

const sortList = ref<RssSort[]>([]);

const currentSort = computed(() => sortList.value[activeSort.value] ?? sortList.value[0]);
const articleStyle = computed(() => Number(source.value?.articleStyle ?? 0));

const menuItems = computed<PopupMenuItem[]>(() => [
  ...(platform.capabilities.webview && String(source.value?.loginUrl || "").trim()
    ? [{ key: "login", label: "登录" }]
    : []),
  { key: "refreshSort", label: "刷新分类缓存" },
  { key: "variables", label: "设置源变量" },
  { key: "editSource", label: "编辑订阅源" },
  { key: "switchLayout", label: "切换布局" },
  { key: "readRecord", label: "阅读记录" },
  { key: "clear", label: "清空文章" },
]);

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2600);
}

async function loadSource() {
  const list = await rssSourceList();
  source.value = list.find((item) => item.sourceUrl === sourceUrl.value) ?? null;
  if (!source.value) throw new Error("订阅源不存在");
  try {
    sortList.value = await rssSortUrls(sourceUrl.value);
  } catch (error) {
    sortList.value = [{ name: "", url: sourceUrl.value }];
    message.value = String(error);
  }
  activeSort.value = 0;
}

async function loadPage(target = 1) {
  if (!source.value || !currentSort.value) return;
  loading.value = true;
  message.value = "";
  try {
    const result = await rssArticles(
      sourceUrl.value,
      currentSort.value.name,
      currentSort.value.url,
      target,
    );
    articles.value = result.articles || [];
    next.value = result.next ?? null;
    page.value = target;
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
    refreshing.value = false;
  }
}

async function refresh() {
  if (refreshing.value) return;
  refreshing.value = true;
  await loadPage(1);
}

async function loadMore() {
  if (loading.value || !next.value) return;
  await loadPage(page.value + 1);
}

function selectSort(index: number) {
  if (activeSort.value === index) return;
  activeSort.value = index;
  articles.value = [];
  next.value = null;
  void loadPage(1);
}

function openArticle(article: RssArticle) {
  void rssReadRecordAdd(article.link, article.title).catch(() => undefined);
  article.read = true;
  router.push({
    path: "/rss-read",
    query: {
      sourceUrl: sourceUrl.value,
      title: article.title,
      link: article.link,
      origin: article.origin || sourceUrl.value,
      sort: currentSort.value?.name || "",
    },
  });
}

async function onMenu(key: string) {
  if (key === "login") {
    if (!platform.capabilities.webview) {
      showMessage("当前运行时不支持 WebView 登录");
      return;
    }
    router.push({ path: "/source-login", query: { type: "rss", url: sourceUrl.value } });
  } else if (key === "refreshSort") {
    await refresh();
  } else if (key === "variables") {
    router.push({ path: "/book-source-variables", query: { type: "rss", url: sourceUrl.value } });
  } else if (key === "editSource") {
    router.push({ path: "/rss-source-edit", query: { sourceUrl: sourceUrl.value } });
  } else if (key === "switchLayout") {
    if (!source.value) return;
    const style = (Number(source.value.articleStyle ?? 0) + 1) % 3;
    source.value = { ...source.value, articleStyle: style };
    await rssSourceSave(source.value);
  } else if (key === "readRecord") {
    readRecordVisible.value = true;
  } else if (key === "clear") {
    if (!confirm("清空当前订阅源文章缓存？")) return;
    await rssArticleClear(sourceUrl.value, null);
    articles.value = [];
    await loadPage(1);
  }
}

function onScroll(event: Event) {
  const element = event.currentTarget as HTMLElement;
  if (element.scrollHeight - element.scrollTop - element.clientHeight < 180) {
    void loadMore();
  }
}

function onPullStart(event: PointerEvent) {
  const element = event.currentTarget as HTMLElement;
  if (element.scrollTop <= 0) {
    pulling = true;
    pullStart = event.clientY;
  }
}

function onPullMove(event: PointerEvent) {
  if (!pulling) return;
  if (event.clientY - pullStart > 80) {
    pulling = false;
    void refresh();
  }
}

function onPullEnd() {
  pulling = false;
}

onMounted(async () => {
  try {
    await loadSource();
    await loadPage(1);
  } catch (error) {
    message.value = String(error);
  }
});
</script>

<template>
  <div class="rss-articles-page">
    <TitleBar :title="source?.sourceName || sourceName" @back="router.push('/rss-sources')">
      <template #actions>
        <button aria-label="刷新" :disabled="refreshing" @click="refresh">
          <Icon name="ic_refresh_black_24dp" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />
    <div v-if="sortList.length > 1" class="tabs">
      <button
        v-for="(sort, index) in sortList"
        :key="`${sort.name}-${sort.url}`"
        :class="{ active: activeSort === index }"
        @click="selectSort(index)"
      >
        {{ sort.name || "全部" }}
      </button>
    </div>

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="refreshing" class="refresh-tip">正在刷新…</p>
    <main
      class="list"
      :class="`style-${articleStyle}`"
      @scroll.passive="onScroll"
      @pointerdown="onPullStart"
      @pointermove="onPullMove"
      @pointerup="onPullEnd"
      @pointercancel="onPullEnd"
    >
      <EmptyState
        v-if="!articles.length && !loading"
        icon="ic_bottom_rss_e"
        text="暂无文章"
      />
      <article
        v-for="(article, index) in articles"
        :key="`${article.origin}-${article.link}-${index}`"
        class="article"
        :class="{ read: article.read }"
        @click="openArticle(article)"
      >
        <div v-if="article.image" class="image-wrap">
          <img :src="article.image" alt="" loading="lazy" />
        </div>
        <div class="text">
          <p class="title">{{ article.title || article.link }}</p>
          <p v-if="article.pubDate" class="date">{{ article.pubDate }}</p>
        </div>
      </article>
      <div v-if="articles.length" class="load-more" @click="loadMore">
        <span v-if="loading">加载中…</span>
        <span v-else-if="!next">没有更多了</span>
        <span v-else>加载更多</span>
      </div>
      <div v-else-if="loading" class="load-more">加载中…</div>
    </main>

    <RssReadRecordDialog v-model="readRecordVisible" />
  </div>
</template>

<style scoped>
.rss-articles-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.tabs {
  display: flex;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
}
.tabs button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 11px 14px;
  font-size: 13px;
  border-bottom: 2px solid transparent;
  white-space: nowrap;
}
.tabs button.active {
  color: var(--ac-primary);
  border-bottom-color: var(--ac-primary);
}
.message,
.refresh-tip {
  margin: 8px 16px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.article {
  display: flex;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ac-divider);
  cursor: pointer;
}
.article:active {
  background: var(--ac-transparent-10);
}
.article .image-wrap {
  flex: none;
  order: 2;
}
.article .image-wrap img {
  width: 110px;
  height: 68px;
  object-fit: cover;
  border-radius: 2px;
}
.article .text {
  flex: 1;
  min-width: 0;
}
.article .title {
  margin: 0;
  color: var(--ac-primary-text);
  font-size: 16px;
  font-weight: 600;
  line-height: 1.35;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.article .date {
  margin: 8px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
  font-style: italic;
}
.article.read .title {
  color: var(--ac-secondary-text);
}
.style-1 .article {
  flex-direction: column;
}
.style-1 .article .image-wrap {
  order: 0;
}
.style-1 .article .image-wrap img {
  width: 100%;
  height: 220px;
}
.style-1 .article .title {
  font-size: 15px;
  padding-top: 4px;
}
.list.style-2 {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-content: start;
}
.style-2 .article {
  flex-direction: column;
  padding: 8px;
  border-bottom: none;
}
.style-2 .article .image-wrap {
  order: 0;
}
.style-2 .article .image-wrap img {
  width: 100%;
  height: 180px;
}
.style-2 .article .title {
  font-size: 13px;
}
.load-more {
  padding: 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 12px;
  cursor: pointer;
}
</style>
