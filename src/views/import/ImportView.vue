<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import {
  localImportDelete,
  localImportImport,
  localImportList,
  localImportScan,
} from "@/api/books";
import { kvGet, kvSet } from "@/api/config";
import type { LocalImportEntry } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const router = useRouter();
const importPath = ref("");
const entries = ref<LocalImportEntry[]>([]);
const selected = ref(new Set<string>());
const query = ref("");
const loading = ref(false);
const scanning = ref(false);
const message = ref("");
const sort = ref(0);
const sortMenuVisible = ref(false);
const moreMenuVisible = ref(false);
const scriptDialogVisible = ref(false);
const nameScript = ref("");

const sortItems = computed<PopupMenuItem[]>(() => [
  { key: "0", label: "按名称", checked: sort.value === 0 },
  { key: "1", label: "按大小", checked: sort.value === 1 },
  { key: "2", label: "按时间", checked: sort.value === 2 },
  { key: "scan", label: "智能扫描文件夹" },
  { key: "script", label: "导入文件名" },
]);

const moreItems: PopupMenuItem[] = [
  { key: "delete", label: "删除选中文件", icon: "ic_outline_delete", danger: true },
];

const visibleEntries = computed(() => {
  const key = query.value.trim().toLowerCase();
  const list = entries.value.filter((item) => !key || item.name.toLowerCase().includes(key));
  return [...list].sort((a, b) => {
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
    if (sort.value === 1) return b.size - a.size;
    if (sort.value === 2) return b.modified - a.modified;
    return a.name.localeCompare(b.name, "zh-CN");
  });
});

const checkableCount = computed(
  () => visibleEntries.value.filter((item) => !item.isDir && item.supported && !item.onShelf).length,
);
const selectedCount = computed(() => selected.value.size);
const allChecked = computed(
  () => checkableCount.value > 0 && selectedCount.value >= checkableCount.value,
);

function formatSize(size: number): string {
  if (!size) return "";
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  return `${(size / 1024 / 1024 / 1024).toFixed(1)} GB`;
}

function formatDate(ts: number): string {
  if (!ts) return "";
  return new Date(ts * 1000).toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

function showToast(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 1800);
}

async function loadDir(path: string) {
  loading.value = true;
  selected.value = new Set();
  try {
    entries.value = await localImportList(path);
    importPath.value = path;
    await kvSet("importBookPath", path);
  } catch (error) {
    showToast(String(error));
  } finally {
    loading.value = false;
  }
}

async function pickFolder() {
  const picked = await open({
    directory: true,
    multiple: false,
    defaultPath: importPath.value || undefined,
    title: "选择文件夹",
  });
  if (typeof picked === "string" && picked) {
    await loadDir(picked);
  }
}

function parentPath(path: string): string {
  const value = path.replace(/[\\/]+$/, "");
  const index = Math.max(value.lastIndexOf("\\"), value.lastIndexOf("/"));
  if (index <= 0) return value;
  return value.slice(0, index);
}

async function goUp() {
  if (!importPath.value) return;
  const parent = parentPath(importPath.value);
  if (parent && parent !== importPath.value) await loadDir(parent);
}

async function openEntry(entry: LocalImportEntry) {
  if (entry.isDir) {
    await loadDir(entry.path);
    return;
  }
  if (entry.onShelf) {
    router.push({ path: "/reader", query: { bookUrl: entry.path } });
    return;
  }
  if (!entry.supported) {
    showToast("暂不支持该格式，当前支持 TXT / EPUB / PDF / UMD / MOBI / AZW3");
    return;
  }
  const next = new Set(selected.value);
  if (next.has(entry.path)) next.delete(entry.path);
  else next.add(entry.path);
  selected.value = next;
}

function toggleAll() {
  if (allChecked.value) {
    selected.value = new Set();
    return;
  }
  selected.value = new Set(
    visibleEntries.value
      .filter((item) => !item.isDir && item.supported && !item.onShelf)
      .map((item) => item.path),
  );
}

function revertSelection() {
  const next = new Set<string>();
  for (const item of visibleEntries.value) {
    if (item.isDir || !item.supported || item.onShelf) continue;
    if (!selected.value.has(item.path)) next.add(item.path);
  }
  selected.value = next;
}

async function importSelected() {
  if (!selectedCount.value) return;
  loading.value = true;
  try {
    const paths = [...selected.value];
    const results = await localImportImport(paths, 0, nameScript.value || null);
    const ok = results.filter((item) => !item.error).length;
    const failed = results.length - ok;
    showToast(failed ? `成功 ${ok} 本，失败 ${failed} 本` : `已添加 ${ok} 本到书架`);
    selected.value = new Set();
    await loadDir(importPath.value);
  } catch (error) {
    showToast(String(error));
  } finally {
    loading.value = false;
  }
}

async function scanFolder() {
  if (!importPath.value || scanning.value) return;
  scanning.value = true;
  try {
    entries.value = await localImportScan(importPath.value);
    selected.value = new Set();
    showToast(`扫描到 ${entries.value.length} 个文件`);
  } catch (error) {
    showToast(String(error));
  } finally {
    scanning.value = false;
  }
}

async function deleteSelected() {
  if (!selectedCount.value) return;
  if (!confirm(`确定删除选中的 ${selectedCount.value} 个文件？`)) return;
  try {
    const count = await localImportDelete([...selected.value]);
    selected.value = new Set();
    await loadDir(importPath.value);
    showToast(`已删除 ${count} 个文件`);
  } catch (error) {
    showToast(String(error));
  }
}

async function onSortMenu(key: string) {
  if (key === "scan") {
    await scanFolder();
    return;
  }
  if (key === "script") {
    scriptDialogVisible.value = true;
    return;
  }
  sort.value = Number(key);
}

async function saveScript() {
  await kvSet("bookImportFileName", nameScript.value);
  scriptDialogVisible.value = false;
  showToast("文件名脚本已保存");
}

function onMore(key: string) {
  if (key === "delete") deleteSelected();
}

onMounted(async () => {
  const [savedPath, savedScript] = await Promise.all([
    kvGet("importBookPath"),
    kvGet("bookImportFileName"),
  ]);
  nameScript.value = savedScript ?? "";
  if (savedPath) await loadDir(savedPath);
});
</script>

<template>
  <div class="import-page">
    <TitleBar title="" :back="true" @back="router.back()">
      <template #content>
        <label class="search-box">
          <Icon name="ic_search_hint" :size="18" />
          <input v-model="query" placeholder="搜索" />
        </label>
      </template>
      <template #actions>
        <button aria-label="选择文件夹" @click="pickFolder">
          <Icon name="ic_folder_open" :size="24" />
        </button>
        <button aria-label="排序" @click="sortMenuVisible = true">
          <Icon name="ic_sort" :size="24" />
        </button>
      </template>
    </TitleBar>

    <div class="path-bar" data-id="lay_top">
      <span class="path" data-id="tv_path">{{ importPath || "/" }}</span>
      <button data-id="tv_go_back" :disabled="!importPath" @click="goUp">上级</button>
    </div>

    <div v-if="loading || scanning" class="progress-bar" data-id="refresh_progress_bar" />
    <p v-if="message" class="toast">{{ message }}</p>

    <main class="file-area">
      <div v-if="visibleEntries.length" class="file-list" data-id="recycler_view">
        <button
          v-for="entry in visibleEntries"
          :key="entry.path"
          class="file-row"
          :class="{ selected: selected.has(entry.path) }"
          @click="openEntry(entry)"
        >
          <span class="row-leading">
            <span v-if="entry.isDir" class="row-icon" data-id="iv_icon">
              <Icon name="ic_folder" :size="26" />
            </span>
            <span v-else-if="entry.onShelf" class="row-icon" data-id="iv_icon">
              <Icon name="ic_book_has" :size="26" />
            </span>
            <span v-else class="checkbox" :class="{ checked: selected.has(entry.path) }" data-id="cb_select">
              <Icon v-if="selected.has(entry.path)" name="ic_check" :size="18" />
            </span>
          </span>
          <span class="row-main">
            <span class="row-name" data-id="tv_name">{{ entry.name }}</span>
            <span v-if="!entry.isDir" class="row-brief" data-id="ll_brief">
              <span class="tag" data-id="tv_tag">{{ (entry.ext || "FILE").toUpperCase() }}</span>
              <span data-id="tv_size">{{ formatSize(entry.size) }}</span>
              <span data-id="tv_date">{{ formatDate(entry.modified) }}</span>
            </span>
          </span>
        </button>
      </div>
      <p v-else class="empty" data-id="tv_empty_msg">
        {{ importPath ? "当前文件夹没有可导入的书籍文件" : "点击右上角文件夹图标选择文件夹" }}
      </p>
    </main>

    <div class="select-bar" data-id="select_action_bar">
      <label class="select-all" @click.prevent="toggleAll">
        <input type="checkbox" :checked="allChecked" readonly />
        <span>{{ allChecked ? "取消全选" : "全选" }}（{{ selectedCount }}/{{ checkableCount }}）</span>
      </label>
      <button class="bar-button" :disabled="!selectedCount" @click="revertSelection">反选</button>
      <button class="bar-button accent" :disabled="!selectedCount" @click="importSelected">
        加入书架
      </button>
      <button class="bar-icon" :disabled="!selectedCount" @click="moreMenuVisible = true">
        <Icon name="ic_more_vert" :size="22" />
      </button>
    </div>

    <PopupMenu v-model="sortMenuVisible" :items="sortItems" :top="54" @select="onSortMenu" />
    <PopupMenu v-model="moreMenuVisible" :items="moreItems" :top="52" :right="10" @select="onMore" />

    <AndroidDialog v-model="scriptDialogVisible" mode="center" title="导入文件名">
      <p class="script-hint">
        使用 JavaScript 处理文件名变量 src，将书名、作者分别赋值给变量 name、author。
      </p>
      <textarea v-model="nameScript" class="script-input" placeholder="// 示例：&#10;name = src.split(' - ')[0]&#10;author = src.split(' - ')[1] || ''" />
      <template #footer>
        <button class="dialog-button" @click="scriptDialogVisible = false">取消</button>
        <button class="dialog-button accent" @click="saveScript">确定</button>
      </template>
    </AndroidDialog>
  </div>
</template>

<style scoped>
.import-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.search-box {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  height: 30px;
  margin: 10px 0;
  padding: 0 10px;
  border-radius: 15px;
  background: rgba(255, 255, 255, 0.16);
  color: inherit;
}
.search-box input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: inherit;
  font-size: 13px;
}
.search-box input::placeholder {
  color: inherit;
  opacity: 0.75;
}
.path-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 36px;
  padding: 8px;
  background: var(--ac-background);
  border-bottom: 1px solid var(--ac-divider);
}
.path {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.path-bar button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  padding: 4px 14px;
  font-size: 13px;
  border-radius: 4px;
}
.path-bar button:disabled {
  opacity: 0.4;
}
.progress-bar {
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--ac-primary), transparent);
  background-size: 50% 100%;
  animation: progress 0.9s linear infinite;
}
@keyframes progress {
  from {
    background-position: -100% 0;
  }
  to {
    background-position: 200% 0;
  }
}
.toast {
  margin: 0;
  padding: 6px 16px;
  font-size: 12px;
  color: var(--ac-primary);
  background: var(--ac-background-menu);
}
.file-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.file-list {
  display: flex;
  flex-direction: column;
}
.file-row {
  display: flex;
  align-items: stretch;
  width: 100%;
  min-height: 60px;
  padding: 0;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
}
.file-row:active,
.file-row.selected {
  background: var(--ac-transparent-10);
}
.row-leading {
  width: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
}
.row-icon {
  display: inline-flex;
  color: var(--ac-primary-text);
}
.checkbox {
  width: 22px;
  height: 22px;
  border: 2px solid var(--ac-secondary-text);
  border-radius: 3px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}
.checkbox.checked {
  border-color: var(--ac-primary);
  background: var(--ac-primary);
}
.row-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 5px 12px 5px 0;
}
.row-name {
  font-size: 16px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.row-brief {
  display: flex;
  align-items: center;
  gap: 15px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.tag {
  padding: 1px 5px;
  border-radius: 2px;
  background: var(--ac-primary);
  color: #fff;
  font-size: 11px;
}
.empty {
  margin: 48px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 14px;
}
.select-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px 6px 16px;
  background: var(--ac-background-menu);
  box-shadow: 0 -2px 2px rgba(0, 0, 0, 0.18);
}
.select-all {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  font-size: 13px;
}
.select-all input {
  accent-color: var(--ac-primary);
}
.bar-button,
.bar-icon {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  min-width: 72px;
  padding: 7px 10px;
  font-size: 13px;
}
.bar-button.accent {
  background: var(--ac-primary);
  color: #fff;
}
.bar-icon {
  min-width: 36px;
  width: 36px;
  height: 36px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-color: transparent;
  color: var(--ac-primary-text);
}
.bar-button:disabled,
.bar-icon:disabled {
  opacity: 0.35;
}
.script-hint {
  margin: 0 0 10px;
  color: var(--ac-secondary-text);
  font-size: 13px;
  line-height: 1.5;
}
.script-input {
  width: 100%;
  min-height: 150px;
  box-sizing: border-box;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px;
  font-family: ui-monospace, monospace;
  font-size: 13px;
}
.dialog-button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  padding: 7px 16px;
  border-radius: 2px;
}
.dialog-button.accent {
  border-color: var(--ac-primary);
  color: var(--ac-primary);
}
</style>
