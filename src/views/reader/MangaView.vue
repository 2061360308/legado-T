<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { bookGet, chapterList } from "@/core/application/books";
import { bookSourceImages } from "@/core/application/bookSources";
import { readerSaveProgress } from "@/core/application/reader";
import type { Book, BookChapter, OnlineImagePage } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const route = useRoute();
const router = useRouter();
const bookUrl = computed(() => String(route.query.bookUrl || ""));
const book = ref<Book | null>(null);
const chapters = ref<BookChapter[]>([]);
const chapterIndex = ref(0);
const page = ref<OnlineImagePage | null>(null);
const loading = ref(false);
const message = ref("");
const zoomed = ref(false);

const currentChapter = computed(() =>
  chapters.value.find((item) => item.index === chapterIndex.value) ?? null,
);

function readableIndex(start: number, direction = 1): number {
  if (!chapters.value.length) return start;
  let index = Math.max(0, Math.min(chapters.value.length - 1, start));
  for (let count = 0; count < chapters.value.length; count += 1) {
    if (!chapters.value[index]?.isVolume) return chapters.value[index].index;
    index += direction;
    if (index < 0) index = chapters.value.length - 1;
    if (index >= chapters.value.length) index = 0;
  }
  return start;
}

async function loadBook() {
  if (!bookUrl.value) return;
  try {
    book.value = await bookGet(bookUrl.value);
    chapters.value = await chapterList(bookUrl.value);
    const fromQuery = Number(route.query.chapter);
    const target = Number.isFinite(fromQuery)
      ? fromQuery
      : book.value?.durChapterIndex ?? 0;
    await loadChapter(readableIndex(target));
  } catch (error) {
    message.value = String(error);
  }
}

async function loadChapter(index: number) {
  if (!bookUrl.value) return;
  const found = chapters.value.find((item) => item.index === index);
  if (!found) return;
  chapterIndex.value = index;
  page.value = null;
  loading.value = true;
  message.value = "";
  try {
    page.value = await bookSourceImages(bookUrl.value, index);
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
  await loadChapter(readableIndex(target, delta > 0 ? 1 : -1));
}

function openToc() {
  router.push({ path: "/toc", query: { bookUrl: bookUrl.value } });
}

onMounted(loadBook);
</script>

<template>
  <div class="manga-page">
    <TitleBar :title="book?.name || '图片阅读'" @back="router.back()">
      <template #actions>
        <button aria-label="目录" @click="openToc">
          <Icon name="ic_toc" :size="24" />
        </button>
        <button aria-label="刷新" @click="loadChapter(chapterIndex)">
          <Icon name="ic_refresh_black_24dp" :size="24" />
        </button>
      </template>
    </TitleBar>

    <p v-if="currentChapter" class="chapter-title">
      {{ currentChapter.title }}
    </p>
    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="message">加载图片中…</p>

    <main class="image-wrap" :class="{ zoomed }" @click="zoomed = !zoomed">
      <template v-if="page?.images.length">
        <img
          v-for="(item, index) in page.images"
          :key="index"
          v-show="item.dataUrl"
          :src="item.dataUrl"
          :alt="page.title"
        />
      </template>
      <p v-else-if="!loading" class="empty">暂无图片</p>
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
.manga-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: #111;
  color: #fff;
}
.chapter-title {
  margin: 0;
  padding: 4px 14px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.72);
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.message,
.empty {
  margin: 8px 14px;
  color: rgba(255, 255, 255, 0.66);
  font-size: 12px;
  text-align: center;
}
.image-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 4px;
}
.image-wrap img {
  max-width: 100%;
  height: auto;
  display: block;
  transition: width 0.15s ease;
}
.image-wrap.zoomed img {
  width: 180%;
  max-width: none;
}
.chapter-nav {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 10px calc(12px + env(safe-area-inset-left)) calc(10px + env(safe-area-inset-bottom));
  border-top: 1px solid rgba(255, 255, 255, 0.12);
  font-size: 12px;
  color: rgba(255, 255, 255, 0.72);
}
.chapter-nav button {
  border: 1px solid rgba(255, 255, 255, 0.24);
  background: transparent;
  color: #fff;
  border-radius: 4px;
  padding: 6px 12px;
  font-size: 12px;
}
</style>
