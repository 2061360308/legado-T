<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import { platform } from "@/runtime";
import { fetchRemoteText } from "@/utils/remoteFetch";
import { saveTextFile } from "@/utils/fileSave";
import {
  bookSourceCheck,
  bookSourceDelete,
  bookSourceImport,
  bookSourceList,
  bookSourceSave,
  bookSourceSetEnabled,
  bookSourceSetExploreEnabled,
} from "@/core/application/bookSources";
import type { BookSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import SearchBar from "@/components/android/SearchBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import ImportRuleDialog from "@/components/android/ImportRuleDialog.vue";
import OnlineImportDialog from "@/views/misc/OnlineImportDialog.vue";
import BookSourceRow from "./BookSourceRow.vue";
import QrCodeDialog from "./QrCodeDialog.vue";
import {
  parseRuleList,
  selectedByState,
  stateByUpdateTime,
  type RuleImportRow,
} from "@/utils/ruleImport";
import { showError } from "@/utils/errors";

type SortKey = "manual" | "auto" | "name" | "url" | "time" | "respond" | "enable";
type GroupMode = "add" | "remove";

interface CheckState {
  message: string;
  checking: boolean;
}

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const sources = ref<BookSource[]>([]);
const keyword = ref("");
const message = ref("");
const loading = ref(false);
const selected = ref(new Set<string>());
const sort = ref<SortKey>("manual");
const sortAscending = ref(true);
const groupSourcesByDomain = ref(false);
const sortVisible = ref(false);
const groupVisible = ref(false);
const topMenuVisible = ref(false);
const moreVisible = ref(false);
const moreSource = ref<BookSource | null>(null);
const moreTop = ref(52);
const moreRight = ref(8);
const checkVisible = ref(false);
const checking = ref(false);
const checkKeyword = ref("斗破");
const checkStates = ref<Record<string, CheckState>>({});
const qrVisible = ref(false);
const onlineVisible = ref(false);
const importVisible = ref(false);
const importRows = ref<RuleImportRow<BookSource>[]>([]);
const importItems = computed(() => importRows.value.map((row) => row.item));
const groupDialogVisible = ref(false);
const groupMode = ref<GroupMode>("add");
const groupName = ref("");

function splitGroups(value?: string | null): string[] {
  return String(value || "")
    .split(/[,，;；]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function hostOf(source: BookSource): string {
  try {
    return new URL(source.bookSourceUrl).hostname || "#";
  } catch {
    return "#";
  }
}

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2500);
}

const groups = computed(() => {
  const values = new Set<string>();
  for (const source of sources.value) {
    for (const group of splitGroups(source.bookSourceGroup)) values.add(group);
  }
  return [...values].sort((a, b) => a.localeCompare(b, "zh-CN"));
});

const filteredByKeyword = computed(() => {
  const key = keyword.value.trim();
  if (!key) return sources.value;
  if (key === "启用") return sources.value.filter((source) => source.enabled);
  if (key === "禁用") return sources.value.filter((source) => !source.enabled);
  if (key === "需要登录") {
    return sources.value.filter((source) => Boolean(String(source.loginUrl || "").trim()));
  }
  if (key === "无分组") {
    return sources.value.filter((source) => !String(source.bookSourceGroup || "").trim());
  }
  if (key === "启用发现") {
    return sources.value.filter(
      (source) =>
        Boolean(String(source.exploreUrl || "").trim()) &&
        source.enabledExplore !== false,
    );
  }
  if (key === "禁用发现") {
    return sources.value.filter(
      (source) =>
        Boolean(String(source.exploreUrl || "").trim()) &&
        source.enabledExplore === false,
    );
  }
  if (key.startsWith("group:")) {
    const group = key.slice(6).trim();
    return sources.value.filter((source) => splitGroups(source.bookSourceGroup).includes(group));
  }
  const lower = key.toLowerCase();
  return sources.value.filter((source) => {
    const name = String(source.bookSourceName || "").toLowerCase();
    const group = String(source.bookSourceGroup || "").toLowerCase();
    return name.includes(lower) || group.includes(lower);
  });
});

function compareSources(a: BookSource, b: BookSource): number {
  switch (sort.value) {
    case "auto":
      return (a.weight ?? 0) - (b.weight ?? 0);
    case "name":
      return String(a.bookSourceName || "").localeCompare(
        String(b.bookSourceName || ""),
        "zh-CN",
      );
    case "url":
      return String(a.bookSourceUrl || "").localeCompare(
        String(b.bookSourceUrl || ""),
      );
    case "time":
      return (b.lastUpdateTime ?? 0) - (a.lastUpdateTime ?? 0);
    case "respond":
      return (a.respondTime ?? 0) - (b.respondTime ?? 0);
    case "enable": {
      const byEnabled = Number(b.enabled) - Number(a.enabled);
      if (byEnabled) return byEnabled;
      return String(a.bookSourceName || "").localeCompare(
        String(b.bookSourceName || ""),
        "zh-CN",
      );
    }
    default:
      return (a.customOrder ?? 0) - (b.customOrder ?? 0);
  }
}

const sortedSources = computed(() => {
  const list = [...filteredByKeyword.value];
  if (groupSourcesByDomain.value) {
    return list.sort((a, b) => {
      const hostA = hostOf(a);
      const hostB = hostOf(b);
      if (hostA === "#" && hostB !== "#") return 1;
      if (hostB === "#" && hostA !== "#") return -1;
      const byHost = hostA.localeCompare(hostB);
      if (byHost) return byHost;
      return (b.lastUpdateTime ?? 0) - (a.lastUpdateTime ?? 0);
    });
  }
  list.sort(compareSources);
  return sortAscending.value ? list : list.reverse();
});

const rows = computed(() => {
  const list = sortedSources.value;
  return list.map((source, index) => {
    const host = hostOf(source);
    return {
      source,
      host,
      showHost:
        groupSourcesByDomain.value &&
        (index === 0 || hostOf(list[index - 1]) !== host),
    };
  });
});

const selectedSources = computed(() =>
  rows.value.filter((row) => selected.value.has(row.source.bookSourceUrl)).map((row) => row.source),
);

const sortItems = computed<PopupMenuItem[]>(() => [
  { key: "manual", label: "手动排序", checked: sort.value === "manual" },
  { key: "auto", label: "权重排序", checked: sort.value === "auto" },
  { key: "name", label: "按名称", checked: sort.value === "name" },
  { key: "url", label: "按 URL", checked: sort.value === "url" },
  { key: "time", label: "按更新时间", checked: sort.value === "time" },
  { key: "respond", label: "按响应时间", checked: sort.value === "respond" },
  { key: "enable", label: "按启用状态", checked: sort.value === "enable" },
  { key: "desc", label: "降序", checked: !sortAscending.value },
]);

const groupItems = computed<PopupMenuItem[]>(() => {
  const key = keyword.value.trim();
  return [
    { key: "filter:all", label: "全部", checked: !key },
    { key: "filter:enabled", label: "启用", checked: key === "启用" },
    { key: "filter:disabled", label: "禁用", checked: key === "禁用" },
    { key: "filter:login", label: "需要登录", checked: key === "需要登录" },
    { key: "filter:none", label: "无分组", checked: key === "无分组" },
    {
      key: "filter:enabledExplore",
      label: "启用发现",
      checked: key === "启用发现",
    },
    {
      key: "filter:disabledExplore",
      label: "禁用发现",
      checked: key === "禁用发现",
    },
    ...groups.value.map((group) => ({
      key: `group:${group}`,
      label: group,
      checked: key === `group:${group}`,
    })),
  ];
});

const topMenuItems = computed<PopupMenuItem[]>(() => [
  { key: "add", label: "新增书源", icon: "ic_add" },
  { key: "importLocal", label: "本地导入", icon: "ic_import" },
  { key: "importOnline", label: "在线导入", icon: "ic_import" },
  { key: "importQr", label: "二维码导入", icon: "ic_scan" },
  {
    key: "toggleDomain",
    label: "按域名分组",
    checked: groupSourcesByDomain.value,
  },
  { key: "help", label: "帮助", icon: "ic_help", muted: true },
]);

const selectionMenuItems: PopupMenuItem[] = [
  { key: "enable", label: "启用" },
  { key: "disable", label: "禁用" },
  { key: "addGroup", label: "添加分组" },
  { key: "removeGroup", label: "移除分组" },
  { key: "enableExplore", label: "启用发现" },
  { key: "disableExplore", label: "禁用发现" },
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "export", label: "导出选中" },
  { key: "share", label: "分享选中" },
  { key: "check", label: "校验选中" },
  { key: "interval", label: "选择区间" },
];

const moreItems = computed<PopupMenuItem[]>(() => {
  const source = moreSource.value;
  if (!source) return [];
  const items: PopupMenuItem[] = [];
  if (sort.value === "manual" && !groupSourcesByDomain.value) {
    items.push({ key: "top", label: "置顶" }, { key: "bottom", label: "置底" });
  }
  if (String(source.loginUrl || "").trim()) {
    items.push({ key: "login", label: "登录" });
  }
  items.push(
    { key: "search", label: "搜索" },
    { key: "debug", label: "调试" },
    { key: "delete", label: "删除", danger: true },
  );
  if (String(source.exploreUrl || "").trim()) {
    items.push({
      key: "explore",
      label: source.enabledExplore === false ? "启用发现" : "关闭发现",
    });
  }
  return items;
});

async function load() {
  loading.value = true;
  try {
    sources.value = await bookSourceList();
    const alive = new Set(sources.value.map((source) => source.bookSourceUrl));
    selected.value = new Set([...selected.value].filter((url) => alive.has(url)));
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

function edit(source?: BookSource) {
  router.push({
    path: "/book-source-edit",
    query: source ? { url: source.bookSourceUrl } : {},
  });
}

function setSelected(source: BookSource, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(source.bookSourceUrl);
  else next.delete(source.bookSourceUrl);
  selected.value = next;
}

function selectAll(value: boolean) {
  selected.value = value
    ? new Set(rows.value.map((row) => row.source.bookSourceUrl))
    : new Set();
}

function revertSelection() {
  const next = new Set<string>();
  for (const row of rows.value) {
    if (!selected.value.has(row.source.bookSourceUrl)) next.add(row.source.bookSourceUrl);
  }
  selected.value = next;
}

function selectInterval() {
  const indexes = rows.value
    .map((row, index) => (selected.value.has(row.source.bookSourceUrl) ? index : -1))
    .filter((index) => index >= 0);
  if (!indexes.length) return;
  const min = Math.min(...indexes);
  const max = Math.max(...indexes);
  const next = new Set(selected.value);
  for (let index = min; index <= max; index += 1) {
    next.add(rows.value[index].source.bookSourceUrl);
  }
  selected.value = next;
}

async function toggle(source: BookSource, value: boolean) {
  try {
    await bookSourceSetEnabled(source.bookSourceUrl, value);
    source.enabled = value;
  } catch (error) {
    showMessage(String(error));
  }
}

async function toggleExplore(source: BookSource) {
  const next = source.enabledExplore === false;
  try {
    await bookSourceSetExploreEnabled(source.bookSourceUrl, next);
    source.enabledExplore = next;
  } catch (error) {
    showMessage(String(error));
  }
}

async function remove(source: BookSource) {
  if (!confirm(`删除书源「${source.bookSourceName || source.bookSourceUrl}」？`)) return;
  try {
    await bookSourceDelete(source.bookSourceUrl);
    selected.value.delete(source.bookSourceUrl);
    await load();
  } catch (error) {
    showMessage(String(error));
  }
}

async function deleteSelected() {
  if (!selectedSources.value.length) return;
  if (!confirm(`删除选中的 ${selectedSources.value.length} 个书源？`)) return;
  try {
    for (const source of selectedSources.value) {
      await bookSourceDelete(source.bookSourceUrl);
    }
    selected.value = new Set();
    await load();
  } catch (error) {
    showMessage(String(error));
  }
}

async function setSelectedEnabled(enabled: boolean) {
  try {
    for (const source of selectedSources.value) {
      if (source.enabled === enabled) continue;
      await bookSourceSetEnabled(source.bookSourceUrl, enabled);
      source.enabled = enabled;
    }
    await load();
  } catch (error) {
    showMessage(String(error));
  }
}

async function setSelectedExplore(enabled: boolean) {
  try {
    for (const source of selectedSources.value) {
      if (!String(source.exploreUrl || "").trim()) continue;
      if (source.enabledExplore === enabled) continue;
      await bookSourceSetExploreEnabled(source.bookSourceUrl, enabled);
      source.enabledExplore = enabled;
    }
    await load();
  } catch (error) {
    showMessage(String(error));
  }
}

async function moveSelected(toTop: boolean) {
  const chosen = selectedSources.value;
  if (!chosen.length) return;
  const selectedUrls = new Set(chosen.map((source) => source.bookSourceUrl));
  const ordered = [...sources.value].sort(
    (a, b) => (a.customOrder ?? 0) - (b.customOrder ?? 0),
  );
  const picked = ordered.filter((source) => selectedUrls.has(source.bookSourceUrl));
  const rest = ordered.filter((source) => !selectedUrls.has(source.bookSourceUrl));
  const next = toTop ? [...picked, ...rest] : [...rest, ...picked];
  try {
    for (let index = 0; index < next.length; index += 1) {
      if ((next[index].customOrder ?? 0) !== index + 1) {
        await bookSourceSave({ ...next[index], customOrder: index + 1 });
      }
    }
    await load();
  } catch (error) {
    showMessage(String(error));
  }
}

async function moveSource(source: BookSource, toTop: boolean) {
  const orders = sources.value.map((item) => item.customOrder ?? 0);
  const order = toTop
    ? Math.min(...orders, 0) - 1
    : Math.max(...orders, 0) + 1;
  try {
    await bookSourceSave({ ...source, customOrder: order });
    await load();
  } catch (error) {
    showMessage(String(error));
  }
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
  try {
    for (const source of selectedSources.value) {
      const values = new Set(splitGroups(source.bookSourceGroup));
      if (groupMode.value === "add") values.add(name);
      else values.delete(name);
      const next = [...values].join(",");
      await bookSourceSave({ ...source, bookSourceGroup: next || null });
    }
    groupDialogVisible.value = false;
    await load();
  } catch (error) {
    showMessage(String(error));
  }
}

async function exportSelected() {
  const list = selectedSources.value;
  if (!list.length) {
    showMessage("请先选择书源");
    return;
  }
  try {
    const saved = await saveTextFile(
      "bookSource.json",
      JSON.stringify(list, null, 2),
      "书源 JSON",
    );
    if (saved) showMessage("书源已导出");
  } catch (error) {
    showMessage(String(error));
  }
}

async function shareSelected() {
  const list = selectedSources.value;
  if (!list.length) return;
  const text = JSON.stringify(list, null, 2);
  if (navigator.share) {
    await navigator.share({ text }).catch(() => undefined);
  } else {
    await navigator.clipboard.writeText(text);
    showMessage("书源 JSON 已复制");
  }
}

function openCheck() {
  if (!selectedSources.value.length) {
    showMessage("请先选择书源");
    return;
  }
  checkVisible.value = true;
}

async function runCheck() {
  if (checking.value) return;
  const urls = selectedSources.value
    .filter((source) => source.enabled && String(source.searchUrl || "").trim())
    .map((source) => source.bookSourceUrl);
  if (!urls.length) {
    checkVisible.value = false;
    showMessage("选中的书源没有可校验的搜索规则");
    return;
  }
  checkVisible.value = false;
  checking.value = true;
  const pending = { ...checkStates.value };
  for (const url of urls) pending[url] = { message: "检查中…", checking: true };
  checkStates.value = pending;
  try {
    const results = await bookSourceCheck(urls, checkKeyword.value.trim() || "斗破");
    const updated = { ...checkStates.value };
    const found = new Set<string>();
    for (const result of results) {
      found.add(result.bookSourceUrl);
      updated[result.bookSourceUrl] = {
        message: result.ok ? `成功 ${result.count} 条` : `失败：${result.error || "未知错误"}`,
        checking: false,
      };
    }
    for (const url of urls) {
      if (!found.has(url)) updated[url] = { message: "失败：无结果", checking: false };
    }
    checkStates.value = updated;
    const passed = results.filter((result) => result.ok).length;
    showMessage(`校验完成：${passed}/${urls.length} 个通过`);
  } catch (error) {
    const updated = { ...checkStates.value };
    for (const url of urls) updated[url] = { message: String(error), checking: false };
    checkStates.value = updated;
    showMessage(String(error));
  } finally {
    checking.value = false;
  }
}

async function openImport(json: string) {
  try {
    const values = parseRuleList<BookSource>(json, ["sources", "bookSources"]);
    if (!values.length) throw new Error("没有可导入的书源");
    const existing = await bookSourceList();
    const byUrl = new Map(existing.map((source) => [source.bookSourceUrl, source]));
    importRows.value = values.map((value, index) => {
      const old = byUrl.get(value.bookSourceUrl);
      const state = stateByUpdateTime(old?.lastUpdateTime, value.lastUpdateTime);
      return {
        value,
        item: {
          key: String(index),
          title: value.bookSourceName || value.bookSourceUrl || `书源 ${index + 1}`,
          subtitle: value.bookSourceUrl,
          state,
          selected: selectedByState(state),
        },
      };
    });
    importVisible.value = true;
  } catch (error) {
    showError("导入书源", error);
  }
}

async function importSelected(keys: string[]) {
  const values = keys
    .map((key) => importRows.value[Number(key)]?.value)
    .filter((value): value is BookSource => Boolean(value));
  if (!values.length) return;
  try {
    const count = await bookSourceImport(JSON.stringify(values));
    await load();
    showMessage(`已导入 ${count} 个书源`);
    importRows.value = [];
  } catch (error) {
    showError("导入书源", error);
  }
}

async function importSource() {
  const path = await platform.dialog.open({
    multiple: false,
    filters: [{ name: "书源 JSON", extensions: ["json", "txt"] }],
  });
  if (typeof path !== "string" || !path) return;
  try {
    const bytes = await platform.files.readFile(path);
    const json = new TextDecoder("utf-8").decode(bytes);
    await openImport(json);
  } catch (error) {
    showError("读取书源文件", error);
  }
}

async function importOnline(url: string) {
  try {
    await openImport(await fetchRemoteText(url));
  } catch (error) {
    showError("在线导入书源", error);
  }
}

async function importQr(text: string) {
  await openImport(text);
}

function onSort(key: string) {
  if (key === "desc") {
    sortAscending.value = !sortAscending.value;
    return;
  }
  sort.value = key as SortKey;
}

function chooseGroup(key: string) {
  const filters: Record<string, string> = {
    "filter:all": "",
    "filter:enabled": "启用",
    "filter:disabled": "禁用",
    "filter:login": "需要登录",
    "filter:none": "无分组",
    "filter:enabledExplore": "启用发现",
    "filter:disabledExplore": "禁用发现",
  };
  keyword.value = key.startsWith("group:") ? key : (filters[key] ?? key);
}

function onTopMenu(key: string) {
  if (key === "add") edit();
  else if (key === "importLocal") importSource();
  else if (key === "importOnline") onlineVisible.value = true;
  else if (key === "importQr") qrVisible.value = true;
  else if (key === "toggleDomain") groupSourcesByDomain.value = !groupSourcesByDomain.value;
  else if (key === "help") showMessage("书源帮助暂未接入");
}

function openMore(source: BookSource, event: MouseEvent) {
  moreSource.value = source;
  moreTop.value = Math.min(event.clientY, window.innerHeight - 260);
  moreRight.value = Math.max(8, window.innerWidth - event.clientX);
  moreVisible.value = true;
}

async function onMore(key: string) {
  const source = moreSource.value;
  if (!source) return;
  if (key === "top") await moveSource(source, true);
  else if (key === "bottom") await moveSource(source, false);
  else if (key === "login") {
    router.push({ path: "/source-login", query: { url: source.bookSourceUrl } });
  } else if (key === "search") {
    router.push({ path: "/book-search", query: { sourceUrl: source.bookSourceUrl } });
  } else if (key === "debug") {
    router.push({ path: "/book-source-debug", query: { url: source.bookSourceUrl } });
  } else if (key === "delete") {
    await remove(source);
  } else if (key === "explore") {
    await toggleExplore(source);
  }
}

async function onSelectionMenu(key: string) {
  if (key === "enable") await setSelectedEnabled(true);
  else if (key === "disable") await setSelectedEnabled(false);
  else if (key === "enableExplore") await setSelectedExplore(true);
  else if (key === "disableExplore") await setSelectedExplore(false);
  else if (key === "top") await moveSelected(true);
  else if (key === "bottom") await moveSelected(false);
  else if (key === "addGroup") openGroupDialog("add");
  else if (key === "removeGroup") openGroupDialog("remove");
  else if (key === "export") await exportSelected();
  else if (key === "share") await shareSelected();
  else if (key === "interval") selectInterval();
  else if (key === "check") openCheck();
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
  <div class="source-page">
    <TitleBar search-only @back="router.back()">
      <template #content>
        <SearchBar v-model="keyword" placeholder="搜索书源" />
      </template>
      <template #actions>
        <button aria-label="排序" @click="sortVisible = true">
          <Icon name="ic_sort" :size="24" />
        </button>
        <button aria-label="分组" @click="groupVisible = true">
          <Icon name="ic_groups" :size="24" />
        </button>
        <button aria-label="更多" @click="topMenuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="sortVisible" :items="sortItems" :top="54" @select="onSort" />
    <PopupMenu v-model="groupVisible" :items="groupItems" :top="54" @select="chooseGroup" />
    <PopupMenu
      v-model="topMenuVisible"
      :items="topMenuItems"
      :top="54"
      @select="onTopMenu"
    />
    <PopupMenu
      v-model="moreVisible"
      :items="moreItems"
      :top="moreTop"
      :right="moreRight"
      @select="onMore"
    />

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else ref="listRef" class="list">
      <EmptyState
        v-if="!rows.length"
        icon="ic_cfg_source"
        text="暂无书源"
        action="新增书源"
        @action="edit()"
      />
      <BookSourceRow
        v-for="row in rows"
        :key="row.source.bookSourceUrl"
        :source="row.source"
        :selected="selected.has(row.source.bookSourceUrl)"
        :host="row.host"
        :show-host="row.showHost"
        :check-message="checkStates[row.source.bookSourceUrl]?.message || ''"
        :checking="checkStates[row.source.bookSourceUrl]?.checking || false"
        @select="setSelected(row.source, $event)"
        @toggle="toggle(row.source, $event)"
        @edit="edit(row.source)"
        @more="openMore(row.source, $event)"
      />
    </main>

    <SelectActionBar
      :count="selectedSources.length"
      :total="rows.length"
      main-text="删除"
      :menu-items="selectionMenuItems"
      @select-all="selectAll"
      @revert="revertSelection"
      @main="deleteSelected"
      @menu="onSelectionMenu"
    />

    <AndroidDialog v-model="checkVisible" mode="center" title="检查书源">
      <label class="form-field">
        搜索关键字
        <input
          v-model="checkKeyword"
          placeholder="搜索关键字"
          @keydown.enter="runCheck"
        />
      </label>
      <p class="form-hint">将检查已选择的 {{ selectedSources.length }} 个书源</p>
      <template #footer>
        <button class="btn" @click="checkVisible = false">取消</button>
        <button class="btn primary" :disabled="checking" @click="runCheck">
          {{ checking ? "检查中…" : "开始检查" }}
        </button>
      </template>
    </AndroidDialog>

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
      title="在线导入书源"
      @confirm="importOnline"
    />
    <QrCodeDialog v-model="qrVisible" mode="import" @import="importQr" />
    <ImportRuleDialog
      v-model="importVisible"
      title="导入书源"
      :items="importItems"
      @confirm="importSelected"
    />
  </div>
</template>

<style scoped>
.source-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.message,
.loading {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
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
.form-hint {
  margin: 10px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
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
.btn:disabled {
  opacity: 0.45;
}
</style>
