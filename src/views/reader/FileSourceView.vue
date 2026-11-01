<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { bookGet, chapterList } from "@/core/application/books";
import { bookSourceResource } from "@/core/application/bookSources";
import { readerSaveProgress } from "@/core/application/reader";
import type { Book, BookChapter, OnlineResource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const route = useRoute();
const router = useRouter();
const bookUrl = computed(() => String(route.query.bookUrl || ""));
const book = ref<Book | null>(null);
const chapters = ref<BookChapter[]>([]);
const chapterIndex = ref(0);
const resource = ref<OnlineResource | null>(null);
const loading = ref(false);
const message = ref("");

const currentChapter = computed(() =>
  chapters.value.find((item) => item.index === chapterIndex.value) ?? null,
);

const fileExtension = computed(() => {
  const value = resource.value?.url || "";
  const match = /\.([a-z0-9]{1,8})(?:[?#]|$)/i.exec(value);
  return match ? match[1].toUpperCase() : "文件";
});

function readableIndex(start: number): number {
  if (!chapters.value.length) return start;
  let index = Math.max(0, Math.min(chapters.value.length - 1, start));
  for (let count = 0; count < chapters.value.length; count += 1) {
    if (!chapters.value[index]?.isVolume) return chapters.value[index].index;
    index = (index + 1) % chapters.value.length;
  }
  return start;
}

async function loadBook() {
  if (!bookUrl.value) return;
  try {
    book.value = await bookGet(bookUrl.value);
    chapters.value = await chapterList(bookUrl.value);
    await loadChapter(readableIndex(book.value?.durChapterIndex ?? 0));
  } catch (error) {
    message.value = String(error);
  }
}

async function loadChapter(index: number) {
  const found = chapters.value.find((item) => item.index === index);
  if (!found) return;
  chapterIndex.value = index;
  resource.value = null;
  loading.value = true;
  message.value = "";
  try {
    resource.value = await bookSourceResource(bookUrl.value, index);
    await readerSaveProgress(bookUrl.value, index, 0, found.title).catch(
      () => undefined,
    );
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function changeChapter(delta: number) {
  const position = chapters.value.findIndex((item) => item.index === chapterIndex.value);
  const target = position + delta;
  if (target < 0 || target >= chapters.value.length) return;
  await loadChapter(readableIndex(target));
}

async function copyUrl() {
  if (!resource.value?.url) return;
  await navigator.clipboard.writeText(resource.value.url);
  message.value = "文件地址已复制";
}

function openFile() {
  if (!resource.value?.url) return;
  window.open(resource.value.url, "_blank", "noopener,noreferrer");
}

function openToc() {
  router.push({ path: "/toc", query: { bookUrl: bookUrl.value } });
}

onMounted(loadBook);
</script>

<template>
  <div class="file-page">
    <TitleBar :title="book?.name || '文件阅读'" @back="router.back()">
      <template #actions>
        <button aria-label="目录" @click="openToc">
          <Icon name="ic_toc" :size="24" />
        </button>
      </template>
    </TitleBar>

    <main class="content">
      <p v-if="currentChapter" class="chapter">{{ currentChapter.title }}</p>
      <p v-if="message" class="message">{{ message }}</p>
      <p v-if="loading" class="message">加载文件地址中…</p>

      <section v-if="resource" class="file-card">
        <span class="badge">{{ fileExtension }}</span>
        <p class="url">{{ resource.url }}</p>
        <div class="row">
          <button @click="openFile">打开文件</button>
          <button @click="copyUrl">复制地址</button>
          <a :href="resource.url" target="_blank" rel="noopener noreferrer">浏览器打开</a>
        </div>
      </section>
      <p v-else-if="!loading" class="empty">该章节没有文件地址</p>
    </main>

    <footer v-if="chapters.length" class="chapter-nav">
      <button :disabled="chapterIndex <= 0" @click="changeChapter(-1)">上一章</button>
      <span>{{ chapterIndex + 1 }} / {{ chapters.length }}</span>
      <button
        :disabled="chapterIndex >= chapters.length - 1"
        @click="changeChapter(1)"
      >
        下一章
      </button>
    </footer>
  </div>
</template>

<style scoped>
.file-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px;
}
.chapter {
  margin: 0 0 12px;
  color: var(--ac-primary-text);
  font-size: 16px;
}
.message,
.empty {
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.file-card {
  border: 1px solid var(--ac-divider);
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.badge {
  align-self: flex-start;
  background: var(--ac-primary);
  color: #fff;
  border-radius: 3px;
  padding: 2px 7px;
  font-size: 11px;
}
.url {
  margin: 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
  word-break: break-all;
  user-select: text;
}
.row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.row button,
.row a {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 7px 10px;
  font-size: 12px;
  text-decoration: none;
}
.row button:first-child {
  background: var(--ac-primary);
  border-color: var(--ac-primary);
  color: #fff;
}
.chapter-nav {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 10px calc(12px + env(safe-area-inset-left)) calc(10px + env(safe-area-inset-bottom));
  border-top: 1px solid var(--ac-divider);
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.chapter-nav button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 6px 12px;
  font-size: 12px;
}
</style>
