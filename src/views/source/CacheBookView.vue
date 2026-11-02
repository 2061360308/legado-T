<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import {
  bookClearCache,
  bookFileSize,
  bookList,
  groupList,
} from "@/core/application/books";
import { onlineReaderDownload } from "@/core/application/reader";
import { kvGet, kvSet } from "@/core/application/config";
import {
  DEFAULT_CACHE_EXPORT_OPTIONS,
  buildEpub,
  buildText,
  formatFileName,
  loadCachedBook,
  type CacheExportOptions,
} from "@/utils/cacheExport";
import { uploadWebdav } from "@/utils/webdav";
import { platform } from "@/runtime";
import type { Book, BookGroup } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import AppLogDialog from "@/views/misc/AppLogDialog.vue";

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const books = ref<{ book: Book; size: number }[]>([]);
const groups = ref<BookGroup[]>([]);
const message = ref("");
const busy = ref("");
const groupFilter = ref(0);
const menuVisible = ref(false);
const logVisible = ref(false);
const groupVisible = ref(false);
const exportOptions = ref<CacheExportOptions>({ ...DEFAULT_CACHE_EXPORT_OPTIONS });
const exportFolder = ref("");

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

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "download", label: "下载全部" },
  { key: "exportAll", label: "导出全部" },
  { key: "replace", label: "替换净化", checked: exportOptions.value.useReplace },
  { key: "custom", label: "自定义导出章节" },
  { key: "webdav", label: "导出到 WebDAV", checked: exportOptions.value.exportToWebdav },
  { key: "noChapter", label: "不导出章节名", checked: exportOptions.value.noChapterName },
  { key: "pics", label: "导出图片", checked: exportOptions.value.exportPictures },
  { key: "parallel", label: "并行导出", checked: exportOptions.value.parallel },
  { key: "folder", label: exportFolder.value ? `导出目录（${exportFolder.value}）` : "导出目录" },
  { key: "filename", label: `导出文件名（${exportOptions.value.filenameTemplate}）` },
  { key: "type", label: `导出类型（${exportOptions.value.type}）` },
  { key: "charset", label: `导出编码（${exportOptions.value.charset}）` },
  { key: "log", label: "日志" },
]);

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

function encodeText(text: string, charset: string): Uint8Array<ArrayBuffer> {
  if (/^utf-?16le$/i.test(charset)) {
    const buffer = new Uint8Array(2 + text.length * 2);
    buffer[0] = 0xff;
    buffer[1] = 0xfe;
    const view = new DataView(buffer.buffer);
    for (let index = 0; index < text.length; index++) {
      view.setUint16(2 + index * 2, text.charCodeAt(index), true);
    }
    return buffer;
  }
  return new TextEncoder().encode(text);
}

async function loadExportOptions() {
  try {
    const raw = await kvGet("cacheExport:options");
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<CacheExportOptions>;
      exportOptions.value = { ...DEFAULT_CACHE_EXPORT_OPTIONS, ...parsed };
    }
    exportFolder.value = (await kvGet("cacheExport:folder")) ?? "";
  } catch {
    exportOptions.value = { ...DEFAULT_CACHE_EXPORT_OPTIONS };
  }
}

async function saveExportOptions() {
  await kvSet("cacheExport:options", JSON.stringify(exportOptions.value));
}

async function chooseExportFolder(): Promise<boolean> {
  const path = await platform.dialog.open({
    directory: true,
    multiple: false,
    title: "选择导出目录",
  });
  if (typeof path !== "string" || !path) return false;
  exportFolder.value = path;
  await kvSet("cacheExport:folder", path);
  return true;
}

async function buildBookBytes(book: Book): Promise<{ bytes: Uint8Array<ArrayBuffer>; name: string; ext: string; mime: string }> {
  const chapters = await loadCachedBook(book, exportOptions.value);
  if (!chapters.length) throw new Error("没有可导出的缓存章节");
  const baseName = formatFileName(exportOptions.value.filenameTemplate, book);
  if (exportOptions.value.type === "epub") {
    return {
      bytes: buildEpub(book, chapters),
      name: `${baseName}.epub`,
      ext: "epub",
      mime: "application/epub+zip",
    };
  }
  const text = buildText(book, chapters, exportOptions.value.noChapterName);
  return {
    bytes: encodeText(text, exportOptions.value.charset),
    name: `${baseName}.txt`,
    ext: "txt",
    mime: "text/plain; charset=utf-8",
  };
}

async function writeExportFile(
  book: Book,
  data: { bytes: Uint8Array<ArrayBuffer>; name: string; ext: string; mime: string },
) {
  if (exportOptions.value.exportToWebdav) {
    const url = await uploadWebdav(data.name, data.bytes, data.mime);
    message.value = `已导出《${book.name}》到 ${url}`;
    return;
  }
  let path: string | null = exportFolder.value
    ? await platform.files.join(exportFolder.value, data.name)
    : "";
  if (!path) {
    path = await platform.dialog.save({
      defaultPath: data.name,
      filters: [
        data.ext === "epub"
          ? { name: "EPUB", extensions: ["epub"] }
          : { name: "文本文件", extensions: ["txt"] },
      ],
    });
  }
  if (!path) return;
  await platform.files.writeFile(path, data.bytes);
  message.value = `已导出《${book.name}》`;
}

async function exportBook(book: Book) {
  if (busy.value) return;
  busy.value = book.bookUrl;
  try {
    await writeExportFile(book, await buildBookBytes(book));
  } catch (error) {
    message.value = String(error);
  } finally {
    busy.value = "";
  }
}

async function exportAll() {
  const selected = filtered.value;
  if (!selected.length) {
    message.value = "没有可导出的书籍";
    return;
  }
  if (!exportOptions.value.exportToWebdav && !exportFolder.value) {
    if (!(await chooseExportFolder())) return;
  }
  busy.value = "all";
  try {
    if (exportOptions.value.parallel) {
      await Promise.all(
        selected.map(async (item) => {
          await writeExportFile(item.book, await buildBookBytes(item.book));
        }),
      );
    } else {
      for (let index = 0; index < selected.length; index++) {
        message.value = `正在导出 ${index + 1}/${selected.length}：《${selected[index].book.name}》`;
        await writeExportFile(selected[index].book, await buildBookBytes(selected[index].book));
      }
    }
    message.value = `已导出 ${selected.length} 本书`;
  } catch (error) {
    message.value = String(error);
  } finally {
    busy.value = "";
  }
}

async function toggleExportOption(key: "useReplace" | "exportToWebdav" | "noChapterName" | "exportPictures" | "parallel") {
  exportOptions.value = { ...exportOptions.value, [key]: !exportOptions.value[key] };
  await saveExportOptions();
}

async function setExportType() {
  const value = window.prompt("导出类型（txt / epub）", exportOptions.value.type);
  if (value !== "txt" && value !== "epub") return;
  exportOptions.value = { ...exportOptions.value, type: value };
  await saveExportOptions();
}

async function setExportCharset() {
  const value = window.prompt("导出编码（UTF-8 / UTF-16LE）", exportOptions.value.charset);
  if (!value) return;
  if (!/^utf-?8$/i.test(value) && !/^utf-?16le$/i.test(value)) {
    message.value = "当前仅支持 UTF-8 和 UTF-16LE";
    return;
  }
  exportOptions.value = {
    ...exportOptions.value,
    charset: /^utf-?16le$/i.test(value) ? "UTF-16LE" : "UTF-8",
  };
  await saveExportOptions();
}

async function setExportFileName() {
  const value = window.prompt(
    "文件名模板，可用 {{name}}、{{author}}",
    exportOptions.value.filenameTemplate,
  );
  if (value === null) return;
  exportOptions.value = { ...exportOptions.value, filenameTemplate: value || "{{name}}" };
  await saveExportOptions();
}

async function setExportScope() {
  const value = window.prompt(
    "导出章节范围，留空导出全部，示例：1-5,8,10-18",
    exportOptions.value.scope,
  );
  if (value === null) return;
  exportOptions.value = { ...exportOptions.value, scope: value.trim() };
  await saveExportOptions();
}

function onMenu(key: string) {
  if (key === "replace") void toggleExportOption("useReplace");
  else if (key === "download") void downloadAll();
  else if (key === "exportAll") void exportAll();
  else if (key === "webdav") void toggleExportOption("exportToWebdav");
  else if (key === "noChapter") void toggleExportOption("noChapterName");
  else if (key === "pics") void toggleExportOption("exportPictures");
  else if (key === "parallel") void toggleExportOption("parallel");
  else if (key === "custom") void setExportScope();
  else if (key === "folder") void chooseExportFolder();
  else if (key === "filename") void setExportFileName();
  else if (key === "type") void setExportType();
  else if (key === "charset") void setExportCharset();
  else if (key === "log") logVisible.value = true;
}

onMounted(async () => {
  await loadExportOptions();
  await load();
});
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
    <main ref="listRef" class="list">
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
    <AppLogDialog v-model="logVisible" />
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
