<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import {
  localImportArchiveImport,
  localImportArchiveList,
  localImportImport,
  localImportReimport,
} from "@/core/application/books";
import type { ArchiveEntry } from "@/core/domain";
import { kvGet, kvSet } from "@/core/application/config";
import type { ImportEntry, ImportLocation, PreparedImport } from "@/platform/contract";
import { platform } from "@/runtime";
import TitleBar from "@/components/android/TitleBar.vue";
import SearchBar from "@/components/android/SearchBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const router = useRouter();
const fileAreaRef = ref<HTMLElement | null>(null);
useScrollMemory(fileAreaRef);
const location = ref<ImportLocation | null>(null);
const stack = ref<{ token: string; name: string }[]>([]);
const entries = ref<ImportEntry[]>([]);
const selected = ref(new Set<string>());
const query = ref("");
const loading = ref(false);
const scanning = ref(false);
const message = ref("");
const sort = ref(0);
const sortMenuVisible = ref(false);
const overflowMenuVisible = ref(false);
const moreMenuVisible = ref(false);
const scriptDialogVisible = ref(false);
const nameScript = ref("");
const charset = ref("");
const charsetDialogVisible = ref(false);
const charsetDraft = ref("");
const CHARSETS = ["", "UTF-8", "GBK", "GB18030", "UTF-16LE", "UTF-16BE", "BIG5"];
const archiveVisible = ref(false);
const archivePath = ref("");
const archiveEntries = ref<ArchiveEntry[]>([]);
const archiveSelected = ref(new Set<string>());
const archiveBusy = ref(false);

const capabilities = computed(() => platform.importPicker.capabilities(location.value));
const currentToken = computed(() => stack.value[stack.value.length - 1]?.token ?? "");
const pathLabel = computed(() => {
  if (!capabilities.value.directoryNavigation) {
    return location.value ? `${location.value.name}（${entries.value.length} 个文件）` : "";
  }
  return stack.value.length ? `${stack.value.map((item) => item.name).join("/")}/` : "";
});
const canGoUp = computed(
  () => capabilities.value.directoryNavigation && stack.value.length > 1,
);

const sortItems = computed<PopupMenuItem[]>(() => [
  { key: "0", label: "按名称", checked: sort.value === 0 },
  { key: "1", label: "按大小", checked: sort.value === 1 },
  { key: "2", label: "按时间", checked: sort.value === 2 },
]);

const overflowItems = computed<PopupMenuItem[]>(() => {
  const items: PopupMenuItem[] = [];
  if (capabilities.value.scan) {
    items.push({ key: "scan", label: "扫描文件夹" });
  }
  items.push({ key: "script", label: "导入文件名" });
  items.push({ key: "encoding", label: "导入编码" });
  return items;
});

const moreItems = computed<PopupMenuItem[]>(() =>
  capabilities.value.delete
    ? [{ key: "delete", label: "删除选中文件", icon: "ic_outline_delete", danger: true }]
    : [],
);

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

function showToast(text: string, duration = 1800) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, duration);
}

async function loadDirectory(token: string): Promise<boolean> {
  if (!location.value) return false;
  loading.value = true;
  selected.value = new Set();
  try {
    entries.value = await platform.importPicker.listDirectory(location.value, token);
    return true;
  } catch (error) {
    entries.value = [];
    showToast(String(error));
    return false;
  } finally {
    loading.value = false;
  }
}

async function applyLocation(next: ImportLocation): Promise<boolean> {
  location.value = next;
  stack.value = [{ token: next.rootToken, name: next.name }];
  await platform.importPicker.saveLocation(next);
  const loaded = await loadDirectory(next.rootToken);
  if (!loaded) {
    location.value = null;
    stack.value = [];
  }
  return loaded;
}

async function chooseFolder() {
  if (loading.value) return;
  try {
    const picked = await platform.importPicker.pickLocation();
    if (picked) await applyLocation(picked);
  } catch (error) {
    showToast(String(error));
  }
}

async function openEntry(entry: ImportEntry) {
  if (entry.isDir) {
    stack.value = [...stack.value, { token: entry.key, name: entry.name }];
    await loadDirectory(entry.key);
    return;
  }
  if (entry.onShelf) {
    if (entry.bookUrl) {
      router.push({ path: "/reader", query: { bookUrl: entry.bookUrl } });
    } else {
      showToast("书籍已在书架");
    }
    return;
  }
  if (entry.ext === "zip") {
    await openArchive(entry);
    return;
  }
  if (!entry.supported) {
    showToast("暂不支持该格式，当前支持 TXT / EPUB / PDF / UMD / MOBI / AZW3 / ZIP");
    return;
  }
  const next = new Set(selected.value);
  if (next.has(entry.key)) next.delete(entry.key);
  else next.add(entry.key);
  selected.value = next;
}

async function goUp() {
  if (!canGoUp.value) return;
  const next = stack.value.slice(0, -1);
  stack.value = next;
  await loadDirectory(next[next.length - 1].token);
}

function toggleAll() {
  if (allChecked.value) {
    selected.value = new Set();
    return;
  }
  selected.value = new Set(
    visibleEntries.value
      .filter((item) => !item.isDir && item.supported && !item.onShelf)
      .map((item) => item.key),
  );
}

function revertSelection() {
  const next = new Set<string>();
  for (const item of visibleEntries.value) {
    if (item.isDir || !item.supported || item.onShelf) continue;
    if (!selected.value.has(item.key)) next.add(item.key);
  }
  selected.value = next;
}

async function reimportEntry(entry: ImportEntry) {
  if (!entry.onShelf || entry.isDir || location.value?.kind !== "desktop") return;
  if (!confirm(`更新《${entry.name}》并重新解析章节？`)) return;
  try {
    const result = await localImportReimport(entry.key, charset.value || null);
    showToast(result.duplicate ? "文件内容未变化" : "已更新书籍并重新解析");
    await loadDirectory(currentToken.value);
  } catch (e) {
    showToast(String(e));
  }
}

async function openArchive(entry: ImportEntry) {
  if (location.value?.kind !== "desktop") {
    showToast("Android SAF 压缩包请先复制到本地目录");
    return;
  }
  try {
    const list = await localImportArchiveList(entry.key);
    archivePath.value = entry.key;
    archiveEntries.value = list.filter((item) => item.supported);
    archiveSelected.value = new Set(archiveEntries.value.map((item) => item.name));
    archiveVisible.value = true;
  } catch (e) {
    showToast(String(e));
  }
}

function toggleArchive(name: string) {
  const next = new Set(archiveSelected.value);
  if (next.has(name)) next.delete(name);
  else next.add(name);
  archiveSelected.value = next;
}

async function importArchive() {
  if (!archivePath.value || archiveBusy.value) return;
  archiveBusy.value = true;
  try {
    const results = await localImportArchiveImport(
      archivePath.value,
      [...archiveSelected.value],
      0,
      nameScript.value || null,
      charset.value || null,
    );
    const ok = results.filter((item) => !item.error && !item.duplicate).length;
    const duplicates = results.filter((item) => item.duplicate).length;
    const failed = results.filter((item) => item.error).length;
    showToast(`压缩包导入：成功 ${ok} 本，失败 ${failed} 本，已存在 ${duplicates} 本`);
    archiveVisible.value = false;
    await loadDirectory(currentToken.value);
  } catch (e) {
    showToast(String(e));
  } finally {
    archiveBusy.value = false;
  }
}

async function importSelected() {
  if (!selectedCount.value) {
    showToast("请先选择要导入的文件");
    return;
  }
  if (loading.value) {
    showToast("正在处理，请稍候");
    return;
  }
  if (!location.value) {
    showToast("请先选择文件夹");
    return;
  }
  loading.value = true;
  let prepared: PreparedImport | undefined;
  try {
    const selectedEntries = entries.value.filter((item) => selected.value.has(item.key));
    prepared = await platform.importPicker.prepareImport(location.value, selectedEntries);
    if (!prepared.paths.length) {
      showToast(prepared.skipped ? `没有可导入的书籍，已跳过 ${prepared.skipped} 个文件` : "没有可导入的书籍");
      return;
    }
    const results = await localImportImport(
      prepared.paths,
      0,
      nameScript.value || null,
      charset.value || null,
    );
    const ok = results.filter((item) => !item.error && !item.duplicate).length;
    const duplicates = results.filter((item) => item.duplicate).length;
    const failed = results.filter((item) => item.error).length;
    const skipped = prepared.skipped ? `，跳过 ${prepared.skipped} 个` : "";
    const duplicateText = duplicates ? `，已存在 ${duplicates} 本` : "";
    const firstError = results.find((item) => item.error)?.error;
    showToast(
      failed
        ? `导入失败：${firstError || `成功 ${ok} 本，失败 ${failed} 本`}${duplicateText}${skipped}`
        : `已添加 ${ok} 本到书架${duplicateText}${skipped}`,
      5000,
    );
    selected.value = new Set();
    await loadDirectory(currentToken.value);
  } catch (error) {
    showToast(String(error), 5000);
  } finally {
    if (prepared) await platform.importPicker.cleanupPreparedImport(prepared);
    loading.value = false;
  }
}

async function scanFolder() {
  if (!capabilities.value.scan || !location.value || !currentToken.value || scanning.value) return;
  scanning.value = true;
  try {
    entries.value = await platform.importPicker.scanDirectory(location.value, currentToken.value);
    selected.value = new Set();
    showToast(`扫描到 ${entries.value.length} 个文件`);
  } catch (error) {
    showToast(String(error));
  } finally {
    scanning.value = false;
  }
}

async function deleteSelected() {
  if (!capabilities.value.delete || !location.value || !selectedCount.value) return;
  if (!confirm(`确定删除选中的 ${selectedCount.value} 个文件？`)) return;
  try {
    const selectedEntries = entries.value.filter((item) => selected.value.has(item.key));
    const count = await platform.importPicker.deleteEntries(location.value, selectedEntries);
    selected.value = new Set();
    await loadDirectory(currentToken.value);
    showToast(`已删除 ${count} 个文件`);
  } catch (error) {
    showToast(String(error));
  }
}

async function onSortMenu(key: string) {
  sort.value = Number(key);
  await kvSet("localBookImportSort", String(sort.value));
}

async function onOverflow(key: string) {
  if (key === "scan") {
    await scanFolder();
  } else if (key === "script") {
    scriptDialogVisible.value = true;
  } else if (key === "encoding") {
    charsetDraft.value = charset.value;
    charsetDialogVisible.value = true;
  }
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
  const savedScript = await kvGet("bookImportFileName");
  nameScript.value = savedScript ?? "";
  const savedSort = Number((await kvGet("localBookImportSort")) ?? 0);
  sort.value = Number.isFinite(savedSort) ? savedSort : 0;
  const saved = await platform.importPicker.loadSavedLocation();
  if (!saved) {
    if (capabilities.value.autoPick) await chooseFolder();
    return;
  }
  const loaded = await applyLocation(saved);
  if (!loaded) showToast("无法读取上次的文件夹");
});
</script>

<template>
  <div class="import-page">
    <TitleBar search-only :back="true" @back="router.back()">
      <template #content>
        <SearchBar v-model="query" placeholder="搜索 本地书籍" />
      </template>
      <template #actions>
        <button aria-label="选择文件夹" @click="chooseFolder">
          <Icon name="ic_folder_open" :size="24" />
        </button>
        <button aria-label="排序" @click="sortMenuVisible = true">
          <Icon name="ic_sort" :size="24" />
        </button>
        <button aria-label="更多" @click="overflowMenuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <div class="path-bar" data-id="lay_top">
      <span class="path" data-id="tv_path">{{ pathLabel || "/" }}</span>
      <button
        v-if="capabilities.directoryNavigation"
        data-id="tv_go_back"
        :disabled="!canGoUp"
        @click="goUp"
      >
        上级
      </button>
    </div>

    <div v-if="loading || scanning" class="progress-bar" data-id="refresh_progress_bar" />
    <p v-if="message" class="toast">{{ message }}</p>

    <main ref="fileAreaRef" class="file-area">
      <div v-if="visibleEntries.length" class="file-list" data-id="recycler_view">
        <button
          v-for="entry in visibleEntries"
          :key="entry.key"
          class="file-row"
          :class="{ selected: selected.has(entry.key) }"
          @click="openEntry(entry)"
          @contextmenu.prevent="reimportEntry(entry)"
        >
          <span class="row-leading">
            <span v-if="entry.isDir" class="row-icon" data-id="iv_icon">
              <Icon name="ic_folder" :size="26" />
            </span>
            <span v-else-if="entry.onShelf" class="row-icon" data-id="iv_icon">
              <Icon name="ic_book_has" :size="26" />
            </span>
            <span v-else class="checkbox" :class="{ checked: selected.has(entry.key) }" data-id="cb_select">
              <Icon v-if="selected.has(entry.key)" name="ic_check" :size="18" />
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
        {{
          pathLabel
            ? "当前列表没有可导入的书籍文件"
            : capabilities.directoryNavigation
              ? "点击右上角文件夹图标选择文件夹"
              : "点击右上角文件夹图标选择文件"
        }}
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
      <button v-if="moreItems.length" class="bar-icon" :disabled="!selectedCount" @click="moreMenuVisible = true">
        <Icon name="ic_more_vert" :size="22" />
      </button>
    </div>

    <PopupMenu v-model="sortMenuVisible" :items="sortItems" :top="54" @select="onSortMenu" />
    <PopupMenu v-model="overflowMenuVisible" :items="overflowItems" :top="54" :right="10" @select="onOverflow" />
    <PopupMenu v-model="moreMenuVisible" :items="moreItems" :top="52" :right="10" @select="onMore" />

    <AndroidDialog
      v-model="charsetDialogVisible"
      mode="center"
      title="导入编码"
      width="min(92vw, 420px)"
    >
      <label class="encoding-field">
        <span>字符集</span>
        <select v-model="charsetDraft">
          <option v-for="item in CHARSETS" :key="item" :value="item">
            {{ item || "自动检测" }}
          </option>
        </select>
      </label>
      <template #footer>
        <button class="dialog-button" @click="charsetDialogVisible = false">取消</button>
        <button
          class="dialog-button accent"
          @click="charset = charsetDraft; charsetDialogVisible = false"
        >
          确定
        </button>
      </template>
    </AndroidDialog>

    <AndroidDialog
      v-model="archiveVisible"
      mode="center"
      title="压缩包内容"
      width="min(92vw, 520px)"
    >
      <div class="archive-list">
        <label v-for="item in archiveEntries" :key="item.name" class="archive-row">
          <input
            type="checkbox"
            :checked="archiveSelected.has(item.name)"
            @change="toggleArchive(item.name)"
          />
          <span class="archive-name">{{ item.name }}</span>
          <span class="archive-size">{{ formatSize(item.size) }}</span>
        </label>
        <p v-if="archiveEntries.length === 0" class="script-hint">压缩包内没有支持的书籍</p>
      </div>
      <template #footer>
        <button class="dialog-button" @click="archiveVisible = false">取消</button>
        <button class="dialog-button accent" :disabled="archiveBusy" @click="importArchive">
          {{ archiveBusy ? "导入中" : "导入" }}
        </button>
      </template>
    </AndroidDialog>

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
.encoding-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.encoding-field select {
  height: 38px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 0 8px;
  font-size: 14px;
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
  max-width: 50px;
  overflow: hidden;
  padding: 1px 5px;
  border-radius: 2px;
  background: var(--ac-primary);
  color: #fff;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
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
.archive-list {
  max-height: 55vh;
  overflow-y: auto;
}
.archive-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 2px;
  border-bottom: 1px solid var(--ac-divider);
  font-size: 13px;
}
.archive-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.archive-size {
  flex: none;
  color: var(--ac-secondary-text);
  font-size: 11px;
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
