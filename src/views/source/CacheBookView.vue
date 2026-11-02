<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import {
  bookClearCache,
  bookFileSize,
  bookList,
  chapterList,
  groupList,
  readChapter,
} from "@/core/application/books";
import { onlineReaderDownload } from "@/core/application/reader";
import { platform } from "@/runtime";
import type { Book, BookGroup } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const books = ref<{ book: Book; size: number }[]>([]);
const groups = ref<BookGroup[]>([]);
const message = ref("");
const busy = ref("");
const groupFilter = ref(0);
const menuVisible = ref(false);
const groupVisible = ref(false);

const groupItems = computed<PopupMenuItem[]>(() => [
  { key: "0", label: "全部", checked: groupFilter.value === 0 },
  ...groups.value
    .filter((group) => group.groupId > 0)
    .map((group) => ({
      key: String(group.groupId),
      label: group.groupName,
      checked: groupFilter.value === group.groupId,
    })),
]);

const menuItems: PopupMenuItem[] = [
  { key: "download", label: "下载全部" },
  { key: "exportAll", label: "导出全部", muted: true },
  { key: "replace", label: "替换净化" },
  { key: "custom", label: "自定义导出章节", muted: true },
  { key: "webdav", label: "导出到 WebDAV", muted: true },
  { key: "noChapter", label: "不导出章节名", muted: true },
  { key: "pics", label: "导出图片", muted: true },
  { key: "parallel", label: "并行导出", muted: true },
  { key: "folder", label: "导出目录", muted: true },
  { key: "filename", label: "导出文件名", muted: true },
  { key: "type", label: "导出类型", muted: true },
  { key: "charset", label: "导出编码", muted: true },
  { key: "log", label: "日志", muted: true },
];

const filtered = computed(() =>
  groupFilter.value
    ? books.value.filter((item) => ((item.book.group ?? 0) & groupFilter.value) !== 0)
    : books.value,
);

function formatSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  return `${(size / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

async function load() {
  const list = await bookList();
  books.value = await Promise.all(
    list.map(async (book) => ({ book, size: await bookFileSize(book.bookUrl) })),
  );
  groups.value = await groupList();
}

async function clearCache(book: Book) {
  if (!confirm(`清除《${book.name}》的离线缓存？`)) return;
  busy.value = book.bookUrl;
  try {
    await bookClearCache(book.bookUrl);
    await load();
    message.value = `《${book.name}》缓存已清除`;
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = "";
  }
}

function isOnlineBook(book: Book): boolean {
  return !book.sourcePath && /^https?:\/\//i.test(String(book.origin || ""));
}

async function downloadBook(book: Book) {
  if (!isOnlineBook(book)) {
    message.value = "本地书籍无需下载";
    return;
  }
  busy.value = book.bookUrl;
  try {
    const count = await onlineReaderDownload(book.bookUrl);
    await load();
    message.value = count
      ? `已下载《${book.name}》${count} 章`
      : `《${book.name}》没有需要下载的章节`;
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = "";
  }
}

async function downloadAll() {
  const selected = filtered.value.filter((item) => isOnlineBook(item.book));
  if (!selected.length) {
    message.value = "没有可下载的在线书籍";
    return;
  }
  busy.value = "download-all";
  let total = 0;
  let failed = 0;
  try {
    for (let i = 0; i < selected.length; i++) {
      const book = selected[i].book;
      message.value = `正在下载 ${i + 1}/${selected.length}：《${book.name}》`;
      try {
        total += await onlineReaderDownload(book.bookUrl);
      } catch {
        failed += 1;
      }
    }
    await load();
    message.value = `已下载 ${total} 章${failed ? `，${failed} 本书失败` : ""}`;
  } finally {
    busy.value = "";
  }
}

function onGroupMenu(key: string) {
  groupFilter.value = Number(key);
}

async function cachedText(book: Book): Promise<string> {
  const chapters = await chapterList(book.bookUrl);
  const parts = [`${book.name}${book.author ? ` / ${book.author}` : ""}`];
  for (const chapter of chapters) {
    try {
      const content = await readChapter(book.bookUrl, chapter.index);
      if (content.trim()) parts.push(`\n${chapter.title}\n\n${content}`);
    } catch {
      // skip chapters that are not cached
    }
  }
  return parts.join("\n");
}

async function exportBook(book: Book) {
  const path = await platform.dialog.save({
    defaultPath: `${book.name || "book"}.txt`,
    filters: [{ name: "文本文件", extensions: ["txt"] }],
  });
  if (!path) return;
  await platform.files.writeFile(path, new TextEncoder().encode(await cachedText(book)));
  message.value = `已导出《${book.name}》`;
}

async function exportAll() {
  const selected = filtered.value;
  if (!selected.length) return;
  const path = await platform.dialog.save({
    defaultPath: "cached-books.txt",
    filters: [{ name: "文本文件", extensions: ["txt"] }],
  });
  if (!path) return;
  busy.value = "all";
  try {
    const parts: string[] = [];
    for (const item of selected) parts.push(await cachedText(item.book));
    await platform.files.writeFile(path, new TextEncoder().encode(parts.join("\n\n")));
    message.value = `已导出 ${selected.length} 本书`;
  } finally {
    busy.value = "";
  }
}

function onMenu(key: string) {
  if (key === "replace") router.push("/replace");
  else if (key === "download") void downloadAll();
  else if (key === "exportAll") void exportAll();
  else if (key === "log") message.value = "日志暂未接入";
  else message.value = "该导出选项暂未接入";
}

onMounted(load);
</script>

<template>
  <div class="cache-page">
    <TitleBar title="离线缓存" @back="router.back()">
      <template #actions>
        <button
          aria-label="下载全部"
          :disabled="busy === 'download-all'"
          @click="downloadAll"
        >
          <Icon name="ic_play_24dp" :size="24" />
        </button>
        <button aria-label="分组" @click="groupVisible = true">
          <Icon name="ic_groups" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>
    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />
    <PopupMenu v-model="groupVisible" :items="groupItems" :top="54" @select="onGroupMenu" />

    <p v-if="message" class="message">{{ message }}</p>
    <main class="list">
      <div v-for="item in filtered" :key="item.book.bookUrl" class="row">
        <div class="text">
          <span class="name">{{ item.book.name }}</span>
          <span class="author">{{ item.book.author || "未知作者" }}</span>
          <span class="status">缓存大小：{{ formatSize(item.size) }}</span>
        </div>
        <button
          class="icon-button"
          aria-label="下载"
          :disabled="busy === item.book.bookUrl"
          @click="downloadBook(item.book)"
        >
          <Icon name="ic_play_24dp" :size="22" />
        </button>
        <button class="export" :disabled="busy === 'all'" @click="exportBook(item.book)">
          导出
        </button>
        <button
          class="icon-button"
          aria-label="清除缓存"
          :disabled="busy === item.book.bookUrl"
          @click="clearCache(item.book)"
        >
          <Icon name="ic_outline_delete" :size="22" />
        </button>
      </div>
      <p v-if="filtered.length === 0" class="empty">暂无可缓存书籍</p>
    </main>
  </div>
</template>

<style scoped>
.cache-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  overflow-y: auto;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name,
.author,
.status {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.name {
  font-size: 15px;
}
.author {
  font-size: 12px;
  color: var(--ac-primary-text);
}
.status {
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.icon-button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.export {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 13px;
  padding: 10px;
}
.export:disabled {
  opacity: 0.4;
}
.empty {
  margin: 40px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
