<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  bookSourceContent,
  bookSourceGet,
  bookSourceInfo,
  bookSourceSearch,
  bookSourceToc,
} from "@/core/application/bookSources";
import type {
  BookSource,
  OnlineBook,
  OnlineChapter,
  OnlineSearchResult,
} from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

interface DebugLog {
  time: string;
  label: string;
  ok: boolean;
  detail: string;
}

const route = useRoute();
const router = useRouter();
const url = computed(() => String(route.query.url || ""));
const source = ref<BookSource | null>(null);
const keyword = ref("斗破");
const running = ref(false);
const message = ref("");
const logs = ref<DebugLog[]>([]);
const results = ref<OnlineSearchResult[]>([]);
const selected = ref<OnlineSearchResult | null>(null);
const info = ref<OnlineBook | null>(null);
const chapters = ref<OnlineChapter[]>([]);
const chapter = ref<OnlineChapter | null>(null);
const content = ref("");

const infoJson = computed(() =>
  info.value ? JSON.stringify(info.value, null, 2) : "",
);
const chapterJson = computed(() =>
  chapter.value ? JSON.stringify(chapter.value, null, 2) : "",
);
const visibleChapters = computed(() => chapters.value.slice(0, 80));

function addLog(label: string, ok: boolean, detail = "") {
  const now = new Date();
  logs.value.unshift({
    time: now.toLocaleTimeString("zh-CN", { hour12: false }),
    label,
    ok,
    detail,
  });
}

function clearResult() {
  results.value = [];
  selected.value = null;
  info.value = null;
  chapters.value = [];
  chapter.value = null;
  content.value = "";
}

async function loadSource() {
  if (!url.value) return;
  try {
    source.value = await bookSourceGet(url.value);
  } catch (error) {
    message.value = String(error);
  }
}

async function runSearch() {
  if (running.value || !url.value) return;
  const text = keyword.value.trim();
  if (!text) {
    message.value = "请输入搜索关键字";
    return;
  }
  running.value = true;
  message.value = "";
  clearResult();
  const started = performance.now();
  try {
    const found = await bookSourceSearch(text, [url.value], 1);
    results.value = found;
    const failures = found.filter((item) => item.error);
    addLog(
      "搜索",
      found.some((item) => !item.error),
      `${found.length} 条，${failures.length} 个错误，${Math.round(performance.now() - started)}ms`,
    );
    if (failures.length) {
      message.value = failures.map((item) => item.error).join("\n");
    }
    const first = found.find((item) => !item.error && item.bookUrl);
    if (first) await runInfo(first);
  } catch (error) {
    addLog("搜索", false, String(error));
    message.value = String(error);
  } finally {
    running.value = false;
  }
}

async function runInfo(result: OnlineSearchResult) {
  if (!result.bookUrl) return;
  selected.value = result;
  info.value = null;
  chapters.value = [];
  chapter.value = null;
  content.value = "";
  const started = performance.now();
  try {
    const book = await bookSourceInfo(url.value, result.bookUrl);
    info.value = book;
    addLog("详情", true, `${Math.round(performance.now() - started)}ms`);
    await runToc();
  } catch (error) {
    addLog("详情", false, String(error));
    message.value = String(error);
  }
}

async function runToc() {
  if (!info.value) return;
  const started = performance.now();
  try {
    const list = await bookSourceToc(url.value, info.value);
    chapters.value = list;
    addLog("目录", true, `${list.length} 章，${Math.round(performance.now() - started)}ms`);
    const first = list.find((item) => !item.isVolume);
    if (first) await runContent(first);
  } catch (error) {
    addLog("目录", false, String(error));
    message.value = String(error);
  }
}

async function runContent(target: OnlineChapter) {
  if (!info.value) return;
  chapter.value = target;
  content.value = "";
  const started = performance.now();
  try {
    const text = await bookSourceContent(url.value, info.value, target);
    content.value = text;
    addLog(
      "正文",
      true,
      `${text.length} 字，${Math.round(performance.now() - started)}ms`,
    );
  } catch (error) {
    addLog("正文", false, String(error));
    message.value = String(error);
  }
}

async function runAll() {
  await runSearch();
}

function editSource() {
  router.push({ path: "/book-source-edit", query: { url: url.value } });
}

function openLogin() {
  router.push({ path: "/source-login", query: { url: url.value } });
}

onMounted(loadSource);
</script>

<template>
  <div class="debug-page">
    <TitleBar title="书源调试" @back="router.back()">
      <template #actions>
        <button aria-label="编辑" @click="editSource">
          <Icon name="ic_edit" :size="24" />
        </button>
      </template>
    </TitleBar>

    <div class="toolbar">
      <span class="source-name">{{ source?.bookSourceName || url || "未选择书源" }}</span>
      <button class="mini" @click="openLogin">登录</button>
    </div>

    <div class="search-row">
      <input v-model="keyword" placeholder="搜索关键字" @keyup.enter="runSearch" />
      <button :disabled="running || !url" @click="runAll">搜索并跑全链路</button>
    </div>

    <p v-if="message" class="message">{{ message }}</p>

    <main class="content">
      <section v-if="results.length" class="panel">
        <h3>搜索结果</h3>
        <div class="result-list">
          <button
            v-for="(item, index) in results"
            :key="`${item.bookUrl}-${index}`"
            class="result"
            :class="{ active: selected?.bookUrl === item.bookUrl }"
            @click="item.error ? undefined : runInfo(item)"
          >
            <span class="result-name">{{ item.error ? "书源失败" : item.name }}</span>
            <span class="result-meta">
              {{ item.error || `${item.author || "未知作者"} · ${item.bookUrl}` }}
            </span>
          </button>
        </div>
      </section>

      <section v-if="info" class="panel">
        <h3>详情</h3>
        <pre>{{ infoJson }}</pre>
      </section>

      <section v-if="chapters.length" class="panel">
        <h3>目录（{{ chapters.length }} 章）</h3>
        <div class="chapter-list">
          <button
            v-for="item in visibleChapters"
            :key="item.index"
            class="chapter"
            :class="{ active: chapter?.index === item.index }"
            @click="runContent(item)"
          >
            <span>{{ item.index + 1 }}. {{ item.title }}</span>
            <small>{{ item.url }}</small>
          </button>
        </div>
      </section>

      <section v-if="chapter" class="panel">
        <h3>章节</h3>
        <pre>{{ chapterJson }}</pre>
      </section>

      <section v-if="content" class="panel">
        <h3>正文（{{ content.length }} 字）</h3>
        <pre class="content-text">{{ content }}</pre>
      </section>

      <section class="panel">
        <h3>日志</h3>
        <p v-if="!logs.length" class="empty">暂无日志</p>
        <div v-for="(item, index) in logs" :key="index" class="log" :class="{ fail: !item.ok }">
          <span class="log-time">{{ item.time }}</span>
          <span class="log-label">{{ item.label }}</span>
          <span class="log-detail">{{ item.detail }}</span>
        </div>
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
.toolbar,
.search-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
}
.source-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.search-row input {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px 10px;
  border-radius: 4px;
}
.search-row button,
.mini {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 6px 10px;
  white-space: nowrap;
}
.message {
  margin: 0 12px 6px;
  padding: 6px 8px;
  border-left: 2px solid var(--ac-primary);
  color: var(--ac-secondary-text);
  font-size: 12px;
  white-space: pre-wrap;
}
.content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 12px 24px;
}
.panel {
  margin-top: 10px;
  border: 1px solid var(--ac-divider);
  border-radius: 4px;
  padding: 8px 10px;
}
.panel h3 {
  margin: 0 0 6px;
  font-size: 13px;
  color: var(--ac-primary-text);
}
pre {
  margin: 0;
  max-height: 320px;
  overflow: auto;
  font-size: 12px;
  color: var(--ac-secondary-text);
  white-space: pre-wrap;
  word-break: break-all;
}
.content-text {
  max-height: 420px;
  color: var(--ac-primary-text);
  font-size: 13px;
  line-height: 1.7;
}
.result-list,
.chapter-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 320px;
  overflow-y: auto;
}
.result,
.chapter {
  display: flex;
  flex-direction: column;
  gap: 2px;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 6px;
  text-align: left;
  border-radius: 3px;
}
.result.active,
.chapter.active {
  background: color-mix(in srgb, var(--ac-primary) 12%, transparent);
}
.chapter small {
  color: var(--ac-secondary-text);
  font-size: 11px;
  word-break: break-all;
}
.result-meta {
  font-size: 11px;
  color: var(--ac-secondary-text);
  word-break: break-all;
}
.log {
  display: flex;
  gap: 8px;
  padding: 3px 0;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.log.fail {
  color: #c0392b;
}
.log-time {
  opacity: 0.7;
}
.log-label {
  min-width: 40px;
}
.log-detail {
  flex: 1;
  word-break: break-all;
}
.empty {
  margin: 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
</style>
