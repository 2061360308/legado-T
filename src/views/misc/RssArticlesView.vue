<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { rssArticles, rssContent, rssSourceList } from "@/core/application/rss";
import type { RssArticle, RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.sourceUrl || ""));
const sourceName = computed(() => String(route.query.sourceName || "订阅"));
const source = ref<RssSource | null>(null);
const articles = ref<RssArticle[]>([]);
const loading = ref(false);
const message = ref("");
const active = ref<RssArticle | null>(null);
const content = ref("");
const contentLoading = ref(false);

function stripHtml(value: string): string {
  const doc = new DOMParser().parseFromString(value, "text/html");
  doc.querySelectorAll("script,style").forEach((node) => node.remove());
  return (doc.body.textContent || "").replace(/\n{3,}/g, "\n\n").trim();
}

async function load() {
  if (!sourceUrl.value) return;
  loading.value = true;
  message.value = "";
  try {
    const list = await rssSourceList();
    source.value = list.find((item) => item.sourceUrl === sourceUrl.value) ?? null;
    const page = await rssArticles(
      sourceUrl.value,
      "",
      String(source.value?.sortUrl || sourceUrl.value),
      1,
    );
    articles.value = page.articles || [];
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function openArticle(article: RssArticle) {
  active.value = article;
  content.value = "";
  if (article.content) {
    content.value = stripHtml(article.content);
    return;
  }
  contentLoading.value = true;
  try {
    const html = await rssContent(
      sourceUrl.value,
      article,
      source.value?.ruleContent ?? null,
    );
    content.value = stripHtml(html) || article.description || "暂无正文";
  } catch (error) {
    content.value = String(error);
  } finally {
    contentLoading.value = false;
  }
}

function closeArticle() {
  active.value = null;
  content.value = "";
}

onMounted(load);
</script>

<template>
  <div class="rss-articles-page">
    <TitleBar :title="sourceName" @back="router.push('/rss-sources')" />
    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else class="list">
      <EmptyState
        v-if="!articles.length"
        icon="ic_bottom_rss_e"
        text="暂无文章"
      />
      <button
        v-for="article in articles"
        :key="article.link || article.title"
        class="article"
        @click="openArticle(article)"
      >
        <span class="title">{{ article.title || article.link }}</span>
        <span v-if="article.pubDate" class="date">{{ article.pubDate }}</span>
        <span v-if="article.description" class="desc">{{ stripHtml(article.description) }}</span>
      </button>
    </main>

    <div v-if="active" class="overlay">
      <header class="overlay-bar">
        <button class="back" @click="closeArticle">返回</button>
        <span class="overlay-title">{{ active.title }}</span>
      </header>
      <article class="content">
        <p v-if="contentLoading">加载中…</p>
        <template v-else>
          <p v-for="(line, index) in content.split('\n')" :key="index">{{ line }}</p>
        </template>
      </article>
    </div>
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
.message,
.loading {
  margin: 8px 16px;
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.article {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 16px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  text-align: left;
  cursor: pointer;
}
.title {
  font-size: 15px;
  color: var(--ac-primary-text);
}
.date {
  font-size: 11px;
  color: var(--ac-secondary-text);
}
.desc {
  font-size: 12px;
  color: var(--ac-secondary-text);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.overlay {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.overlay-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  color: var(--ac-primary-text);
}
.back {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 14px;
}
.overlay-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
}
.content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px 20px 40px;
  color: var(--ac-primary-text);
  font-size: 17px;
  line-height: 1.8;
  white-space: pre-wrap;
}
.content p {
  margin: 0 0 0.6em;
}
</style>
