<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import {
  rssSourceDelete,
  rssSourceImport,
  rssSourceList,
  rssSourceSave,
} from "@/core/application/rss";
import type { RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import QrCodeDialog from "@/views/source/QrCodeDialog.vue";

const router = useRouter();
const sources = ref<RssSource[]>([]);
const keyword = ref("");
const loading = ref(false);
const message = ref("");
const selected = ref(new Set<string>());
const menuVisible = ref(false);
const groupVisible = ref(false);
const moreVisible = ref(false);
const moreSource = ref<RssSource | null>(null);
const qrVisible = ref(false);
const dragIndex = ref(-1);
const importOnlineVisible = ref(false);
const importOnlineUrl = ref("");
const importInput = ref<HTMLInputElement | null>(null);

const menuItems: PopupMenuItem[] = [
  { key: "add", label: "新建订阅源" },
  { key: "importLocal", label: "本地导入" },
  { key: "importOnline", label: "在线导入" },
  { key: "importQr", label: "二维码导入" },
  { key: "importDefault", label: "导入默认规则" },
  { key: "help", label: "帮助" },
];

const groupItems = computed<PopupMenuItem[]>(() => [
  { key: "enabled", label: "启用" },
  { key: "disabled", label: "禁用" },
  { key: "login", label: "需要登录" },
  { key: "none", label: "无分组" },
  ...groups.value.map((group) => ({
    key: `group:${group}`,
    label: group,
    checked: keyword.value.trim() === `group:${group}`,
  })),
]);

const groups = computed(() => {
  const values = new Set<string>();
  for (const source of sources.value) {
    for (const group of String(source.sourceGroup || "").split(/[,，;；]/)) {
      const name = group.trim();
      if (name) values.add(name);
    }
  }
  return [...values].sort((a, b) => a.localeCompare(b, "zh-CN"));
});

const filtered = computed(() => {
  const key = keyword.value.trim();
  if (!key) return sources.value;
  if (key === "启用") return sources.value.filter((source) => source.enabled !== false);
  if (key === "禁用") return sources.value.filter((source) => source.enabled === false);
  if (key === "需要登录") {
    return sources.value.filter((source) => Boolean(String(source.loginUrl || "").trim()));
  }
  if (key === "无分组") {
    return sources.value.filter((source) => !String(source.sourceGroup || "").trim());
  }
  if (key.startsWith("group:")) {
    const group = key.slice(6).trim();
    return sources.value.filter((source) =>
      String(source.sourceGroup || "")
        .split(/[,，;；]/)
        .map((value) => value.trim())
        .includes(group),
    );
  }
  const lower = key.toLowerCase();
  return sources.value.filter((source) => {
    const name = String(source.sourceName || "").toLowerCase();
    const group = String(source.sourceGroup || "").toLowerCase();
    return name.includes(lower) || group.includes(lower);
  });
});

const selectedSources = computed(() =>
  filtered.value.filter((source) => selected.value.has(source.sourceUrl)),
);

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2600);
}

async function load() {
  loading.value = true;
  try {
    sources.value = await rssSourceList();
    const alive = new Set(sources.value.map((source) => source.sourceUrl));
    selected.value = new Set(
      [...selected.value].filter((url) => alive.has(url)),
    );
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

function displayName(source: RssSource): string {
  const group = String(source.sourceGroup || "").trim();
  return group
    ? `${source.sourceName || source.sourceUrl} (${group})`
    : source.sourceName || source.sourceUrl;
}

function toggleSelected(source: RssSource, checked: boolean) {
  const next = new Set(selected.value);
  if (checked) next.add(source.sourceUrl);
  else next.delete(source.sourceUrl);
  selected.value = next;
}

function selectAll() {
  selected.value = new Set(filtered.value.map((source) => source.sourceUrl));
}

function invertSelection() {
  const next = new Set<string>();
  for (const source of filtered.value) {
    if (!selected.value.has(source.sourceUrl)) next.add(source.sourceUrl);
  }
  selected.value = next;
}

async function setEnabled(enabled: boolean) {
  for (const source of selectedSources.value) {
    await rssSourceSave({ ...source, enabled });
  }
  await load();
}

async function addGroup() {
  const group = window.prompt("分组名称");
  if (!group?.trim()) return;
  for (const source of selectedSources.value) {
    const values = new Set(
      String(source.sourceGroup || "")
        .split(/[,，;；]/)
        .map((value) => value.trim())
        .filter(Boolean),
    );
    values.add(group.trim());
    await rssSourceSave({ ...source, sourceGroup: [...values].join(",") });
  }
  await load();
}

async function removeGroup() {
  const group = window.prompt("要移除的分组名称");
  if (!group?.trim()) return;
  for (const source of selectedSources.value) {
    const values = String(source.sourceGroup || "")
      .split(/[,，;；]/)
      .map((value) => value.trim())
      .filter((value) => value && value !== group.trim());
    await rssSourceSave({ ...source, sourceGroup: values.join(",") || null });
  }
  await load();
}

async function moveSelected(direction: "top" | "bottom") {
  const urls = new Set(selectedSources.value.map((source) => source.sourceUrl));
  if (!urls.size) return;
  const ordered = [...sources.value].sort(
    (a, b) => (a.customOrder ?? 0) - (b.customOrder ?? 0),
  );
  const chosen = ordered.filter((source) => urls.has(source.sourceUrl));
  const rest = ordered.filter((source) => !urls.has(source.sourceUrl));
  const next = direction === "top" ? [...chosen, ...rest] : [...rest, ...chosen];
  for (let index = 0; index < next.length; index++) {
    const source = next[index];
    if ((source.customOrder ?? 0) !== index + 1) {
      await rssSourceSave({ ...source, customOrder: index + 1 });
    }
  }
  await load();
}

async function exportSelected() {
  const list = selectedSources.value.length ? selectedSources.value : sources.value;
  if (!list.length) {
    showMessage("暂无可导出的订阅源");
    return;
  }
  const path = await platform.dialog.save({
    defaultPath: "rss-sources.json",
    filters: [{ name: "RSS JSON", extensions: ["json"] }],
  });
  if (!path) return;
  await platform.files.writeFile(
    path,
    new TextEncoder().encode(JSON.stringify(list, null, 2)),
  );
  showMessage("订阅源已导出");
}

async function shareSelected() {
  const list = selectedSources.value.length ? selectedSources.value : sources.value;
  if (!list.length) return;
  const text = JSON.stringify(list, null, 2);
  if (navigator.share) {
    await navigator.share({ text }).catch(() => undefined);
  } else {
    await navigator.clipboard.writeText(text);
    showMessage("订阅源 JSON 已复制");
  }
}

function selectInterval() {
  const indexes = filtered.value
    .map((source, index) => (selected.value.has(source.sourceUrl) ? index : -1))
    .filter((index) => index >= 0);
  if (!indexes.length) return;
  const min = Math.min(...indexes);
  const max = Math.max(...indexes);
  const next = new Set(selected.value);
  for (let index = min; index <= max; index++) {
    next.add(filtered.value[index].sourceUrl);
  }
  selected.value = next;
}

async function deleteSelected() {
  if (!selected.value.size) return;
  if (!confirm(`删除选中的 ${selected.value.size} 个订阅源？`)) return;
  for (const source of selectedSources.value) {
    await rssSourceDelete(source.sourceUrl);
  }
  selected.value = new Set();
  await load();
}

async function edit(source: RssSource) {
  router.push({ path: "/rss-source-edit", query: { sourceUrl: source.sourceUrl } });
}

async function toggleEnabled(source: RssSource, enabled: boolean) {
  await rssSourceSave({ ...source, enabled });
  await load();
}

async function remove(source: RssSource) {
  if (!confirm(`删除订阅源「${source.sourceName || source.sourceUrl}」？`)) return;
  await rssSourceDelete(source.sourceUrl);
  await load();
}

async function moveSource(source: RssSource, direction: "top" | "bottom") {
  const ordered = [...sources.value].sort(
    (a, b) => (a.customOrder ?? 0) - (b.customOrder ?? 0),
  );
  const index = ordered.findIndex((item) => item.sourceUrl === source.sourceUrl);
  if (index < 0) return;
  const [item] = ordered.splice(index, 1);
  if (direction === "top") ordered.unshift(item);
  else ordered.push(item);
  for (let i = 0; i < ordered.length; i++) {
    await rssSourceSave({ ...ordered[i], customOrder: i + 1 });
  }
  await load();
}

async function onLocalFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const count = await rssSourceImport(await file.text());
    await load();
    showMessage(`已导入 ${count} 个订阅源`);
  } catch (error) {
    showMessage(String(error));
  } finally {
    input.value = "";
  }
}

async function importOnline() {
  const url = importOnlineUrl.value.trim();
  if (!url) return;
  try {
    const response = await platform.http.fetch(url);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const count = await rssSourceImport(await response.text());
    importOnlineVisible.value = false;
    importOnlineUrl.value = "";
    await load();
    showMessage(`已导入 ${count} 个订阅源`);
  } catch (error) {
    showMessage(String(error));
  }
}

async function importQr(text: string) {
  try {
    const count = await rssSourceImport(text);
    await load();
    showMessage(`二维码导入 ${count} 个订阅源`);
  } catch (error) {
    showMessage(String(error));
  }
}

async function onMenu(key: string) {
  if (key === "add") router.push("/rss-source-edit");
  else if (key === "importLocal") importInput.value?.click();
  else if (key === "importOnline") importOnlineVisible.value = true;
  else if (key === "importQr") qrVisible.value = true;
  else if (key === "importDefault") showMessage("默认订阅源暂未内置");
  else if (key === "help") showMessage("订阅源规则帮助暂未接入");
}

async function onMore(key: string) {
  const source = moreSource.value;
  if (!source) return;
  if (key === "top") await moveSource(source, "top");
  else if (key === "bottom") await moveSource(source, "bottom");
  else if (key === "delete") await remove(source);
}

function onGroupMenu(key: string) {
  if (key === "enabled") keyword.value = "启用";
  else if (key === "disabled") keyword.value = "禁用";
  else if (key === "login") keyword.value = "需要登录";
  else if (key === "none") keyword.value = "无分组";
  else keyword.value = key;
}

function onDragStart(index: number) {
  dragIndex.value = index;
}

async function onDrop(index: number) {
  const from = dragIndex.value;
  dragIndex.value = -1;
  if (from < 0 || from === index) return;
  const ordered = [...filtered.value];
  const [item] = ordered.splice(from, 1);
  ordered.splice(index, 0, item);
  for (let i = 0; i < ordered.length; i++) {
    await rssSourceSave({ ...ordered[i], customOrder: i + 1 });
  }
  await load();
}

onMounted(load);
</script>

<template>
  <div class="manage-page">
    <TitleBar title="订阅源" @back="router.back()">
      <template #content>
        <label class="search">
          <input v-model="keyword" type="search" placeholder="搜索订阅源" />
        </label>
      </template>
      <template #actions>
        <button aria-label="新增" @click="router.push('/rss-source-edit')">
          <Icon name="ic_add" :size="24" />
        </button>
        <button aria-label="分组" @click="groupVisible = true">
          <Icon name="ic_groups" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />
    <PopupMenu v-model="groupVisible" :items="groupItems" @select="onGroupMenu" />
    <PopupMenu
      v-model="moreVisible"
      :items="[
        { key: 'top', label: '置顶' },
        { key: 'bottom', label: '置底' },
        { key: 'delete', label: '删除', danger: true },
      ]"
      @select="onMore"
    />
    <input
      ref="importInput"
      class="hidden-input"
      type="file"
      accept="application/json,.json,.txt"
      @change="onLocalFile"
    />

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else class="list">
      <div
        v-for="(source, index) in filtered"
        :key="source.sourceUrl"
        class="row"
        draggable="true"
        @dragstart="onDragStart(index)"
        @dragover.prevent
        @drop="onDrop(index)"
      >
        <input
          type="checkbox"
          :checked="selected.has(source.sourceUrl)"
          @change="toggleSelected(source, ($event.target as HTMLInputElement).checked)"
        />
        <span class="name">{{ displayName(source) }}</span>
        <input
          class="switch"
          type="checkbox"
          :checked="source.enabled !== false"
          @change="toggleEnabled(source, ($event.target as HTMLInputElement).checked)"
        />
        <button class="icon-button" aria-label="编辑" @click="edit(source)">
          <Icon name="ic_edit" :size="22" />
        </button>
        <button
          class="icon-button"
          aria-label="更多"
          @click="moreSource = source; moreVisible = true"
        >
          <Icon name="ic_more_vert" :size="22" />
        </button>
      </div>
      <p v-if="!filtered.length" class="empty">暂无订阅源</p>
    </main>

    <div v-if="selected.size" class="selection-bar">
      <span class="count">已选择 {{ selected.size }} 项</span>
      <button @click="selectAll">全选</button>
      <button @click="invertSelection">反选</button>
      <button @click="setEnabled(true)">启用</button>
      <button @click="setEnabled(false)">禁用</button>
      <button @click="addGroup">添加分组</button>
      <button @click="removeGroup">移除分组</button>
      <button @click="moveSelected('top')">置顶</button>
      <button @click="moveSelected('bottom')">置底</button>
      <button @click="exportSelected">导出</button>
      <button @click="shareSelected">分享</button>
      <button @click="selectInterval">选择区间</button>
      <button class="danger" @click="deleteSelected">删除</button>
      <button @click="selected = new Set()">取消</button>
    </div>

    <div v-if="importOnlineVisible" class="modal-mask" @click.self="importOnlineVisible = false">
      <section class="modal">
        <h3>在线导入</h3>
        <input v-model="importOnlineUrl" placeholder="https://example.com/rss-sources.json" />
        <div class="modal-actions">
          <button @click="importOnlineVisible = false">取消</button>
          <button @click="importOnline">导入</button>
        </div>
      </section>
    </div>

    <QrCodeDialog v-model="qrVisible" mode="import" @import="importQr" />
  </div>
</template>

<style scoped>
.manage-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.search {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
}
.search input {
  width: 100%;
  height: 30px;
  border: none;
  border-radius: 15px;
  background: rgba(255, 255, 255, 0.22);
  color: #fff;
  font-size: 13px;
  padding: 0 14px;
  outline: none;
}
.hidden-input {
  display: none;
}
.message,
.loading {
  margin: 8px 16px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--ac-divider);
}
.name {
  flex: 1;
  min-width: 0;
  color: var(--ac-primary-text);
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.switch {
  width: 20px;
  height: 20px;
}
.icon-button {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 50%;
}
.icon-button:active {
  background: var(--ac-transparent-10);
}
.empty {
  padding: 24px;
  text-align: center;
  color: var(--ac-secondary-text);
}
.selection-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  overflow-x: auto;
  background: var(--ac-background-menu);
  border-top: 1px solid var(--ac-divider);
}
.selection-bar .count {
  flex: none;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.selection-bar button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 12px;
  padding: 6px;
}
.selection-bar button.danger {
  color: var(--ac-error);
}
.modal-mask {
  position: fixed;
  inset: 0;
  z-index: 70;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}
.modal {
  width: min(100%, 420px);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 18px;
  border-radius: 4px;
}
.modal h3 {
  margin: 0 0 12px;
  font-size: 16px;
}
.modal input {
  width: 100%;
  box-sizing: border-box;
  padding: 9px 10px;
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: inherit;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}
</style>
