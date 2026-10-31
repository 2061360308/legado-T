<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { platform } from "@/runtime";
import {
  bookClearCache,
  bookCoverRefresh,
  bookDelete,
  bookFileSize,
  bookGet,
  bookMoveToTop,
  bookSetGroup,
  bookUpsert,
  chapterList,
  chapterWordCounts,
  groupList,
} from "@/core/application/books";
import type { Book, BookChapter, BookGroup } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import GroupSelectDialog from "./GroupSelectDialog.vue";

const route = useRoute();
const router = useRouter();
const bookUrl = computed(() => String(route.query.bookUrl || ""));
const book = ref<Book | null>(null);
const chapters = ref<BookChapter[]>([]);
const groups = ref<BookGroup[]>([]);
const fileSize = ref(0);
const wordCount = ref(0);
const menuVisible = ref(false);
const groupDialogVisible = ref(false);
const message = ref("");

const displayCover = computed(() => book.value?.customCoverUrl || book.value?.coverUrl || "");
const displayIntro = computed(() => book.value?.customIntro || book.value?.intro || "暂无简介");
const backgroundSrc = computed(() => {
  const src = displayCover.value;
  if (!src) return "";
  if (/^(https?:|data:|asset:|blob:)/i.test(src)) return src;
  try {
    return platform.assets.fileSrc(src);
  } catch {
    return src;
  }
});
function hasGroupBit(value: number, bit: number): boolean {
  if (!bit) return false;
  return Math.floor(value / bit) % 2 === 1;
}

const groupName = computed(() => {
  const value = book.value?.group ?? 0;
  const names = groups.value
    .filter((group) => hasGroupBit(value, group.groupId))
    .map((group) => group.groupName);
  if (names.length) return names.join("、");
  return book.value?.type && (book.value.type & 16) ? "本地未分组" : "未分组";
});
const kindTags = computed(() => {
  const tags = (book.value?.kind ?? "")
    .split(/[,;，；]/)
    .map((value) => value.trim())
    .filter(Boolean);
  if (wordCount.value > 0) tags.push(`${wordCount.value.toLocaleString("zh-CN")} 字`);
  if (fileSize.value > 0) tags.push(formatSize(fileSize.value));
  return tags;
});
const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "edit", label: "编辑书籍信息", icon: "ic_edit" },
  { key: "share", label: "分享", icon: "ic_share" },
  { key: "refresh", label: "刷新", icon: "ic_refresh_black_24dp" },
  { key: "top", label: "置顶", icon: "ic_book_last" },
  { key: "copy", label: "复制书籍地址", icon: "ic_share" },
  { key: "copyToc", label: "复制目录地址", icon: "ic_share" },
  { key: "canUpdate", label: "允许更新", checked: book.value?.canUpdate === true },
  { key: "splitLong", label: "拆分超长章节" },
  { key: "cache", label: "清除缓存", icon: "ic_outline_delete" },
  { key: "log", label: "日志", muted: true },
]);

function formatSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  return `${(size / 1024 / 1024 / 1024).toFixed(1)} GB`;
}

function showToast(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 1800);
}

async function load() {
  if (!bookUrl.value) return;
  book.value = await bookGet(bookUrl.value);
  if (!book.value) return;
  chapters.value = await chapterList(bookUrl.value);
  groups.value = (await groupList()).filter((group) => group.groupId > 0);
  fileSize.value = await bookFileSize(bookUrl.value);
  try {
    const counts = await chapterWordCounts(bookUrl.value);
    wordCount.value = counts.reduce((sum, item) => sum + item.count, 0);
  } catch {
    wordCount.value = 0;
  }
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

async function onMenu(key: string) {
  if (key === "edit") edit();
  else if (key === "share") {
    const text = `${book.value?.name || ""} ${book.value?.author || ""}`.trim();
    if (navigator.share) await navigator.share({ title: book.value?.name, text }).catch(() => undefined);
    else {
      await navigator.clipboard.writeText(text);
      showToast("已复制书籍信息");
    }
  } else if (key === "refresh") {
    await load();
    showToast("已刷新");
  } else if (key === "top") {
    await bookMoveToTop(bookUrl.value);
    showToast("已置顶");
  } else if (key === "copy") {
    await navigator.clipboard.writeText(bookUrl.value);
    showToast("已复制书籍地址");
  } else if (key === "copyToc") {
    await navigator.clipboard.writeText(book.value?.tocUrl || bookUrl.value);
    showToast("已复制目录地址");
  } else if (key === "canUpdate") {
    if (!book.value) return;
    book.value.canUpdate = !book.value.canUpdate;
    await bookUpsert(book.value);
    showToast(book.value.canUpdate ? "已允许更新" : "已禁止更新");
  } else if (key === "splitLong") {
    if (!book.value) return;
    book.value.readConfig = {
      ...(book.value.readConfig ?? ({} as NonNullable<Book["readConfig"]>)),
      splitLongChapter: !(book.value.readConfig?.splitLongChapter ?? false),
    };
    await bookUpsert(book.value);
    showToast(book.value.readConfig.splitLongChapter ? "已开启拆分超长章节" : "已关闭拆分超长章节");
  } else if (key === "cache") {
    try {
      await bookClearCache(bookUrl.value);
      showToast("缓存已清除");
    } catch (e) {
      showToast(String(e));
    }
  } else if (key === "log") {
    showToast("日志暂未接入");
  }
}

async function removeFromShelf() {
  if (!book.value) return;
  if (!confirm(`删除《${book.value.name}》及应用内文件？`)) return;
  await bookDelete(bookUrl.value, true);
  router.back();
}

async function saveGroup(groupId: number) {
  try {
    await bookSetGroup(bookUrl.value, groupId);
    groupDialogVisible.value = false;
    showToast("分组已更新");
  } catch (error) {
    showToast(String(error));
  }
}

watch(groupDialogVisible, (open) => {
  if (!open) load();
});

onMounted(load);
</script>

<template>
  <div v-if="book" class="book-info">
    <div
      class="bg"
      :style="backgroundSrc ? { backgroundImage: `url(${backgroundSrc})` } : undefined"
    />
    <div class="bg-mask" />
    <TitleBar title="书籍信息" dark @back="router.back()">
      <template #actions>
        <button aria-label="编辑" @click="edit">
          <Icon name="ic_edit" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <main class="scroll">
      <div class="hero">
        <div class="arc" />
        <div class="cover-wrap" @click="edit">
          <CoverImage
            :name="book.name"
            :author="book.author"
            :src="displayCover"
            :width="110"
            :height="160"
            :radius="5"
          />
        </div>
      </div>

      <section class="info">
        <p class="name">{{ book.name }}</p>
        <div v-if="kindTags.length" class="kind-row">
          <span v-for="tag in kindTags" :key="tag" class="kind-tag">{{ tag }}</span>
        </div>
        <p class="line">
          <Icon name="ic_author" :size="18" />
          <span>作者：{{ book.author || "未知作者" }}</span>
        </p>
        <p class="line">
          <Icon name="ic_web_outline" :size="18" />
          <span>来源：{{ book.originName || book.origin || "本地" }}</span>
        </p>
        <p class="line">
          <Icon name="ic_book_last" :size="18" />
          <span>最新：{{ book.latestChapterTitle || "暂无最新章节" }}</span>
        </p>
        <p class="line">
          <Icon name="ic_groups" :size="18" />
          <span>分组：{{ groupName }}</span>
          <button class="inline-action" @click="groupDialogVisible = true">分组设置</button>
        </p>
        <p class="line">
          <Icon name="ic_toc" :size="18" />
          <span>目录：{{ book.durChapterTitle || "尚未开始阅读" }}</span>
          <button class="inline-action" @click="toc">查看目录</button>
        </p>
        <p class="intro">{{ displayIntro }}</p>
      </section>
    </main>

    <footer class="actions">
      <button @click="removeFromShelf">删除书籍</button>
      <button class="primary" @click="read">阅读</button>
    </footer>

    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />

    <GroupSelectDialog
      v-model="groupDialogVisible"
      :group-id="book.group ?? 0"
      @confirm="saveGroup"
    />

    <p v-if="message" class="toast">{{ message }}</p>
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
  background-size: cover;
  background-position: center;
  filter: blur(1px);
  transform: scale(1.04);
}
.bg-mask {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.55), rgba(0, 0, 0, 0.15) 45%, var(--ac-background) 78%);
  pointer-events: none;
}
:deep(.title-bar) {
  position: relative;
  z-index: 2;
  background: transparent;
  box-shadow: none;
  color: #fff;
}
.scroll {
  position: relative;
  z-index: 1;
  flex: 1;
  overflow-y: auto;
}
.hero {
  position: relative;
  min-height: 218px;
  display: flex;
  justify-content: center;
  padding-top: 8px;
}
.arc {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 92px;
  background: var(--ac-background);
  border-radius: 50% 50% 0 0 / 32px 32px 0 0;
}
.cover-wrap {
  position: relative;
  z-index: 1;
  margin-top: 8px;
  padding: 3px;
  background: var(--ac-background);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
}
.info {
  background: var(--ac-background);
  padding: 0 18px 24px;
}
.name {
  margin: 6px 0;
  text-align: center;
  font-size: 18px;
  color: var(--ac-primary-text);
}
.kind-row {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}
.kind-tag {
  padding: 2px 7px;
  border-radius: 2px;
  background: var(--ac-primary);
  color: #fff;
  font-size: 12px;
}
.line {
  margin: 4px 0;
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  font-size: 13px;
  color: var(--ac-summary-text);
}
.line span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.inline-action {
  margin-left: auto;
  flex: none;
  border: none;
  background: var(--ac-transparent-10);
  color: var(--ac-primary);
  padding: 4px 8px;
  font-size: 12px;
  border-radius: 2px;
}
.intro {
  margin: 14px 0 0;
  font-size: 14px;
  line-height: 1.7;
  color: var(--ac-secondary-text);
  white-space: pre-wrap;
  user-select: text;
}
.actions {
  position: relative;
  z-index: 2;
  flex: none;
  display: flex;
  border-top: 1px solid var(--ac-divider);
  background: var(--ac-background-menu);
}
.actions button {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 15px 10px calc(15px + env(safe-area-inset-bottom));
  font-size: 15px;
  cursor: pointer;
}
.actions button.primary {
  background: var(--ac-primary);
  color: #fff;
}
.toast {
  position: fixed;
  left: 50%;
  bottom: 84px;
  transform: translateX(-50%);
  margin: 0;
  padding: 8px 14px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.72);
  color: #fff;
  font-size: 13px;
  z-index: 20;
}
.loading {
  display: flex;
  height: 100%;
  align-items: center;
  justify-content: center;
  color: var(--ac-secondary-text);
}
</style>
