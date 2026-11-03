<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import { fetchRemoteText } from "@/utils/remoteFetch";
import { saveTextFile } from "@/utils/fileSave";
import {
  rssSourceDelete,
  rssSourceImport,
  rssSourceList,
  rssSourceSave,
} from "@/core/application/rss";
import type { RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import SearchBar from "@/components/android/SearchBar.vue";
import Icon from "@/components/android/Icon.vue";
import RuleRow from "@/components/android/RuleRow.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import ImportRuleDialog from "@/components/android/ImportRuleDialog.vue";
import OnlineImportDialog from "@/views/misc/OnlineImportDialog.vue";
import QrCodeDialog from "@/views/source/QrCodeDialog.vue";
import defaultRssSources from "@/assets/defaultData/rssSources.json";
import {
  parseRuleList,
  selectedByState,
  stateByUpdateTime,
  type RuleImportRow,
} from "@/utils/ruleImport";
import { showError } from "@/utils/errors";

type GroupMode = "add" | "remove";

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const sources = ref<RssSource[]>([]);
const keyword = ref("");
const loading = ref(false);
const message = ref("");
const selected = ref(new Set<string>());
const menuVisible = ref(false);
const groupVisible = ref(false);
const moreVisible = ref(false);
const moreSource = ref<RssSource | null>(null);
const moreTop = ref(52);
const moreRight = ref(8);
const qrVisible = ref(false);
const onlineVisible = ref(false);
const importVisible = ref(false);
const importRows = ref<RuleImportRow<RssSource>[]>([]);
const importItems = computed(() => importRows.value.map((row) => row.item));
const importInput = ref<HTMLInputElement | null>(null);
const groupDialogVisible = ref(false);
const groupMode = ref<GroupMode>("add");
const groupName = ref("");

const menuItems: PopupMenuItem[] = [
  { key: "add", label: "新建订阅源", icon: "ic_add" },
  { key: "importLocal", label: "本地导入", icon: "ic_import" },
  { key: "importOnline", label: "在线导入", icon: "ic_import" },
  { key: "importQr", label: "二维码导入", icon: "ic_scan" },
  { key: "importDefault", label: "导入默认规则", icon: "ic_import" },
  { key: "help", label: "帮助", icon: "ic_help", muted: true },
];

const selectionMenuItems: PopupMenuItem[] = [
  { key: "enable", label: "启用" },
  { key: "disable", label: "禁用" },
  { key: "addGroup", label: "添加分组" },
  { key: "removeGroup", label: "移除分组" },
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "export", label: "导出选中" },
  { key: "share", label: "分享选中" },
  { key: "interval", label: "选择区间" },
];

const moreItems: PopupMenuItem[] = [
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "delete", label: "删除", danger: true },
];

const groups = computed(() => {
  const values = new Set<string>();
  for (const source of sources.value) {
    for (const group of splitGroups(source.sourceGroup)) values.add(group);
  }
  return [...values].sort((a, b) => a.localeCompare(b, "zh-CN"));
});

const groupItems = computed<PopupMenuItem[]>(() => {
  const key = keyword.value.trim();
  return [
    { key: "all", label: "全部", checked: !key },
    { key: "enabled", label: "启用", checked: key === "启用" },
    { key: "disabled", label: "禁用", checked: key === "禁用" },
    { key: "login", label: "需要登录", checked: key === "需要登录" },
    { key: "none", label: "无分组", checked: key === "无分组" },
    ...groups.value.map((group) => ({
      key: `group:${group}`,
      label: group,
      checked: key === `group:${group}`,
    })),
  ];
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
    return sources.value.filter((source) => splitGroups(source.sourceGroup).includes(group));
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

function splitGroups(value?: string | null): string[] {
  return String(value || "")
    .split(/[,，;；]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function displayName(source: RssSource): string {
  const group = String(source.sourceGroup || "").trim();
  return group
    ? `${source.sourceName || source.sourceUrl} (${group})`
    : source.sourceName || source.sourceUrl;
}

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
    selected.value = new Set([...selected.value].filter((url) => alive.has(url)));
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

function setSelected(source: RssSource, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(source.sourceUrl);
  else next.delete(source.sourceUrl);
  selected.value = next;
}

function selectAll(value: boolean) {
  selected.value = value
    ? new Set(filtered.value.map((source) => source.sourceUrl))
    : new Set();
}

function revertSelection() {
  selected.value = new Set(
    filtered.value.filter((source) => !selected.value.has(source.sourceUrl)).map((source) => source.sourceUrl),
  );
}

async function setEnabled(enabled: boolean) {
  for (const source of selectedSources.value) {
    if ((source.enabled !== false) === enabled) continue;
    await rssSourceSave({ ...source, enabled });
  }
  await load();
}

function openGroupDialog(mode: GroupMode) {
  if (!selectedSources.value.length) return;
  groupMode.value = mode;
  groupName.value = "";
  groupDialogVisible.value = true;
}

async function applyGroup() {
  const name = groupName.value.trim();
  if (!name) return;
  for (const source of selectedSources.value) {
    const values = new Set(splitGroups(source.sourceGroup));
    if (groupMode.value === "add") values.add(name);
    else values.delete(name);
    await rssSourceSave({
      ...source,
      sourceGroup: [...values].join(",") || null,
    });
  }
  groupDialogVisible.value = false;
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
  for (let index = 0; index < next.length; index += 1) {
    const source = next[index];
    if ((source.customOrder ?? 0) !== index + 1) {
      await rssSourceSave({ ...source, customOrder: index + 1 });
    }
  }
  await load();
}

async function exportSelected() {
  if (!selectedSources.value.length) return;
  const saved = await saveTextFile(
    "rss-sources.json",
    JSON.stringify(selectedSources.value, null, 2),
    "RSS JSON",
  );
  if (saved) showMessage("订阅源已导出");
}

async function shareSelected() {
  if (!selectedSources.value.length) return;
  const text = JSON.stringify(selectedSources.value, null, 2);
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
  for (let index = min; index <= max; index += 1) {
    next.add(filtered.value[index].sourceUrl);
  }
  selected.value = next;
}

async function deleteSelected() {
  if (!selectedSources.value.length) return;
  if (!confirm(`删除选中的 ${selectedSources.value.length} 个订阅源？`)) return;
  for (const source of selectedSources.value) {
    await rssSourceDelete(source.sourceUrl);
  }
  selected.value = new Set();
  await load();
}

function edit(source: RssSource) {
  router.push({ path: "/rss-source-edit", query: { sourceUrl: source.sourceUrl } });
}

async function toggleEnabled(source: RssSource, enabled: boolean) {
  await rssSourceSave({ ...source, enabled });
  await load();
}

async function remove(source: RssSource) {
  if (!confirm(`删除订阅源「${source.sourceName || source.sourceUrl}」？`)) return;
  await rssSourceDelete(source.sourceUrl);
  selected.value.delete(source.sourceUrl);
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
  for (let index = 0; index < ordered.length; index += 1) {
    await rssSourceSave({ ...ordered[index], customOrder: index + 1 });
  }
  await load();
}

async function openImport(json: string) {
  try {
    const values = parseRuleList<RssSource>(json, ["sources", "rssSources"]);
    if (!values.length) throw new Error("没有可导入的订阅源");
    const existing = await rssSourceList();
    const byUrl = new Map(existing.map((source) => [source.sourceUrl, source]));
    importRows.value = values.map((value, index) => {
      const old = byUrl.get(value.sourceUrl);
      const state = stateByUpdateTime(old?.lastUpdateTime, value.lastUpdateTime);
      return {
        value,
        item: {
          key: String(index),
          title: value.sourceName || value.sourceUrl || `订阅源 ${index + 1}`,
          subtitle: value.sourceUrl,
          state,
          selected: selectedByState(state),
        },
      };
    });
    importVisible.value = true;
  } catch (error) {
    showError("导入订阅源", error);
  }
}

async function importSelected(keys: string[]) {
  const values = keys
    .map((key) => importRows.value[Number(key)]?.value)
    .filter((value): value is RssSource => Boolean(value));
  if (!values.length) return;
  try {
    const count = await rssSourceImport(JSON.stringify(values));
    await load();
    showMessage(`已导入 ${count} 个订阅源`);
    importRows.value = [];
  } catch (error) {
    showError("导入订阅源", error);
  }
}

async function onLocalFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    await openImport(await file.text());
  } catch (error) {
    showError("读取订阅源文件", error);
  } finally {
    input.value = "";
  }
}

async function importOnline(url: string) {
  try {
    await openImport(await fetchRemoteText(url));
  } catch (error) {
    showError("在线导入订阅源", error);
  }
}

async function importQr(text: string) {
  await openImport(text);
}

function onMenu(key: string) {
  if (key === "add") router.push("/rss-source-edit");
  else if (key === "importLocal") importInput.value?.click();
  else if (key === "importOnline") onlineVisible.value = true;
  else if (key === "importQr") qrVisible.value = true;
  else if (key === "importDefault") importDefault();
  else if (key === "help") showMessage("订阅源规则帮助暂未接入");
}

async function importDefault() {
  await openImport(JSON.stringify(defaultRssSources));
}

function openMore(source: RssSource, event: MouseEvent) {
  moreSource.value = source;
  moreTop.value = Math.min(event.clientY, window.innerHeight - 180);
  moreRight.value = Math.max(8, window.innerWidth - event.clientX);
  moreVisible.value = true;
}

async function onMore(key: string) {
  const source = moreSource.value;
  if (!source) return;
  if (key === "top") await moveSource(source, "top");
  else if (key === "bottom") await moveSource(source, "bottom");
  else if (key === "delete") await remove(source);
}

function onGroupMenu(key: string) {
  if (key === "all") keyword.value = "";
  else if (key === "enabled") keyword.value = "启用";
  else if (key === "disabled") keyword.value = "禁用";
  else if (key === "login") keyword.value = "需要登录";
  else if (key === "none") keyword.value = "无分组";
  else keyword.value = key;
}

async function onSelectionMenu(key: string) {
  if (key === "enable") await setEnabled(true);
  else if (key === "disable") await setEnabled(false);
  else if (key === "addGroup") openGroupDialog("add");
  else if (key === "removeGroup") openGroupDialog("remove");
  else if (key === "top") await moveSelected("top");
  else if (key === "bottom") await moveSelected("bottom");
  else if (key === "export") await exportSelected();
  else if (key === "share") await shareSelected();
  else if (key === "interval") selectInterval();
}

let activatedOnce = false;
onMounted(async () => {
  await load();
  activatedOnce = true;
});
onActivated(() => {
  if (activatedOnce) void load();
});
</script>

<template>
  <div class="manage-page">
    <TitleBar search-only @back="router.back()">
      <template #content>
        <SearchBar v-model="keyword" placeholder="搜索订阅源" />
      </template>
      <template #actions>
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
      :items="moreItems"
      :top="moreTop"
      :right="moreRight"
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
    <main v-else ref="listRef" class="list">
      <RuleRow
        v-for="source in filtered"
        :key="source.sourceUrl"
        :name="displayName(source)"
        :enabled="source.enabled !== false"
        :selected="selected.has(source.sourceUrl)"
        @update:selected="setSelected(source, $event)"
        @toggle="toggleEnabled(source, $event)"
        @edit="edit(source)"
        @more="openMore(source, $event)"
      />
      <p v-if="!filtered.length" class="empty">暂无订阅源</p>
    </main>

    <SelectActionBar
      :count="selectedSources.length"
      :total="filtered.length"
      main-text="删除"
      :menu-items="selectionMenuItems"
      @select-all="selectAll"
      @revert="revertSelection"
      @main="deleteSelected"
      @menu="onSelectionMenu"
    />

    <AndroidDialog
      v-model="groupDialogVisible"
      mode="center"
      :title="groupMode === 'add' ? '添加分组' : '移除分组'"
    >
      <label class="form-field">
        分组名称
        <input v-model="groupName" placeholder="分组名称" @keydown.enter="applyGroup" />
      </label>
      <template #footer>
        <button class="btn" @click="groupDialogVisible = false">取消</button>
        <button class="btn primary" @click="applyGroup">确定</button>
      </template>
    </AndroidDialog>

    <OnlineImportDialog
      v-model="onlineVisible"
      title="在线导入订阅源"
      @confirm="importOnline"
    />
    <QrCodeDialog v-model="qrVisible" mode="import" @import="importQr" />
    <ImportRuleDialog
      v-model="importVisible"
      title="导入订阅源"
      :items="importItems"
      @confirm="importSelected"
    />
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
.empty {
  padding: 24px;
  text-align: center;
  color: var(--ac-secondary-text);
}
.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
}
.form-field input {
  width: 100%;
  box-sizing: border-box;
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: var(--ac-background);
  color: var(--ac-primary-text);
  outline: none;
}
.btn {
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px 16px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
</style>
