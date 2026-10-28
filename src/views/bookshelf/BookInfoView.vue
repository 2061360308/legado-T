<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { bookGet, chapterList } from "@/api/books";
import type { Book, BookChapter } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const route = useRoute();
const router = useRouter();
const bookUrl = computed(() => String(route.query.bookUrl || ""));
const book = ref<Book | null>(null);
const chapters = ref<BookChapter[]>([]);

async function load() {
  if (!bookUrl.value) return;
  book.value = await bookGet(bookUrl.value);
  chapters.value = await chapterList(bookUrl.value);
}

function read() {
  router.push({ path: "/reader", query: { bookUrl: bookUrl.value } });
}

function toc() {
  router.push({ path: "/toc", query: { bookUrl: bookUrl.value } });
}

function edit() {
  router.push({ path: "/book-info-edit", query: { bookUrl: bookUrl.value } });
}

onMounted(load);
</script>

<template>
  <div v-if="book" class="book-info">
    <div class="bg" />
    <TitleBar :title="book.name" dark @back="router.back()">
      <template #actions>
        <button aria-label="编辑" @click="edit">
          <Icon name="ic_edit" :size="24" />
        </button>
      </template>
    </TitleBar>

    <main class="scroll">
      <div class="cover-wrap">
        <CoverImage :name="book.name" :width="110" :height="160" :radius="5" />
      </div>
      <div class="info">
        <p class="name">{{ book.name }}</p>
        <p class="line">
          <Icon name="ic_author" :size="18" />
          <span>{{ book.author || "未知作者" }}</span>
        </p>
        <p class="line">
          <Icon name="ic_web_outline" :size="18" />
          <span>{{ book.originName || book.origin || "本地" }}</span>
        </p>
        <p class="line">
          <Icon name="ic_toc" :size="18" />
          <span>{{ chapters.length }} 章 · {{ book.charset || "UTF-8" }}</span>
        </p>
        <p class="label">简介</p>
        <p class="intro">{{ book.intro || "暂无简介" }}</p>
      </div>
    </main>

    <div class="actions">
      <button @click="toc">目录</button>
      <button class="primary" @click="read">开始阅读</button>
    </div>
  </div>
  <div v-else class="loading">加载中…</div>
</template>

<style scoped>
.book-info {
  position: relative;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
  overflow: hidden;
}
.bg {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.45), var(--ac-background) 220px);
  pointer-events: none;
}
:deep(.title-bar) {
  position: relative;
  z-index: 1;
}
.scroll {
  position: relative;
  z-index: 1;
  flex: 1;
  overflow-y: auto;
}
.cover-wrap {
  display: flex;
  justify-content: center;
  margin-top: 8px;
}
.info {
  background: var(--ac-background);
  padding: 8px 18px 24px;
}
.name {
  margin: 6px 0;
  text-align: center;
  font-size: 18px;
  color: var(--ac-primary-text);
}
.line {
  margin: 3px 0;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--ac-summary-text);
}
.label {
  margin: 14px 0 6px;
  font-size: 13px;
  color: var(--ac-primary);
}
.intro {
  margin: 0;
  font-size: 13px;
  line-height: 1.7;
  color: var(--ac-secondary-text);
  white-space: pre-wrap;
}
.actions {
  flex: none;
  display: flex;
  gap: 10px;
  padding: 10px 16px calc(10px + env(safe-area-inset-bottom));
  border-top: 1px solid var(--ac-divider);
  background: var(--ac-background);
}
.actions button {
  flex: 1;
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 10px;
  font-size: 14px;
  cursor: pointer;
}
.actions button.primary {
  background: var(--ac-primary);
  color: #fff;
}
.loading {
  display: flex;
  height: 100%;
  align-items: center;
  justify-content: center;
  color: var(--ac-secondary-text);
}
</style>
