<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  rssArticles,
  rssContent,
  rssSourceList,
} from "@/core/application/rss";
import type { RssArticle, RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.sourceUrl || ""));
const source = ref<RssSource | null>(null);
const articles = ref<RssArticle[]>([]);
const active = ref<RssArticle | null>(null);
const content = ref("");
const loading = ref(false);
const message = ref("");

function sortUrl(): string {
  const raw = String(source.value?.sortUrl || "").trim();
  if (!raw || raw.startsWith("@js:") || raw.startsWith("<js>")) return sourceUrl.value;
  const first = raw.split(/(?:&&|\n)+/).find(Boolean) || "";
  const url = first.includes("::") ? first.split("::")[1] : first;
  return url || sourceUrl.value;
}

async function load() {
  loading.value = true;
  try {
    source.value =
      (await rssSourceList()).find((item) => item.sourceUrl === sourceUrl.value) ?? null;
    if (!source.value) throw new Error("订阅源不存在");
    const page = await rssArticles(sourceUrl.value, "", sortUrl(), 1);
    articles.value = page.articles || [];
    if (articles.value[0]) await inspect(articles.value[0]);
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function inspect(article: RssArticle) {
  active.value = article;
  content.value = "";
  try {
    content.value = await rssContent(
      sourceUrl.value,
      article,
      source.value?.ruleContent ?? null,
    );
  } catch (error) {
    content.value = String(error);
  }
}

onMounted(load);
</script>

<template>
  <div class="debug-page">
    <TitleBar title="订阅源调试" @back="router.back()" />
    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="status">加载中…</p>
    <main v-else class="debug-body">
      <section class="articles">
        <h3>文章列表</h3>
        <button
          v-for="article in articles"
          :key="article.link"
          :class="{ active: active?.link === article.link }"
          @click="inspect(article)"
        >
          {{ article.title || article.link }}
        </button>
      </section>
      <section class="detail">
        <h3>{{ active?.title || "正文源码" }}</h3>
        <pre>{{ content }}</pre>
      </section>
    </main>
  </div>
</template>

<style scoped>
.debug-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.message,
.status {
  margin: 8px 16px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.debug-body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
}
.articles {
  width: 38%;
  min-width: 0;
  overflow-y: auto;
  border-right: 1px solid var(--ac-divider);
  padding: 8px;
}
.articles h3,
.detail h3 {
  margin: 4px 6px 8px;
  color: var(--ac-secondary-text);
  font-size: 12px;
  font-weight: normal;
}
.articles button {
  display: block;
  width: 100%;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
  padding: 8px 6px;
  font-size: 13px;
  border-radius: 3px;
}
.articles button.active {
  background: var(--ac-transparent-10);
}
.detail {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  padding: 8px;
}
.detail pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--ac-primary-text);
  font-size: 12px;
  line-height: 1.5;
}
</style>
