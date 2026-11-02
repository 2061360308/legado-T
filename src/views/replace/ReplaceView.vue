<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import type { ReplaceRule } from "@/core/domain";
import {
  replaceRuleDelete,
  replaceRuleImport,
  replaceRuleSave,
  replaceRules,
} from "@/core/application/rules";
import { readerInvalidateAll } from "@/core/application/reader";
import { fetchRemoteText } from "@/utils/remoteFetch";
import { saveTextFile } from "@/utils/fileSave";
import TitleBar from "@/components/android/TitleBar.vue";
import SearchBar from "@/components/android/SearchBar.vue";
import Icon from "@/components/android/Icon.vue";
import RuleRow from "@/components/android/RuleRow.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";
import OnlineImportDialog from "@/views/misc/OnlineImportDialog.vue";
import QrCodeDialog from "@/views/source/QrCodeDialog.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const rules = ref<ReplaceRule[]>([]);
const keyword = ref("");
const groupFilter = ref("");
const selected = ref(new Set<number>());
const message = ref("");
const menuVisible = ref(false);
const groupMenuVisible = ref(false);
const moreVisible = ref(false);
const moreRule = ref<ReplaceRule | null>(null);
const moreTop = ref(52);
const moreRight = ref(8);
const onlineVisible = ref(false);
const qrVisible = ref(false);
const importInput = ref<HTMLInputElement | null>(null);

const menuItems: PopupMenuItem[] = [
  { key: "add", label: "添加替换规则", icon: "ic_add" },
  { key: "importLocal", label: "本地导入", icon: "ic_import" },
  { key: "importOnline", label: "在线导入", icon: "ic_import" },
  { key: "importQr", label: "二维码导入", icon: "ic_scan" },
  { key: "help", label: "帮助", icon: "ic_help", muted: true },
];

const selectionMenuItems: PopupMenuItem[] = [
  { key: "enable", label: "启用" },
  { key: "disable", label: "禁用" },
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "export", label: "导出选中" },
];

const moreItems: PopupMenuItem[] = [
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "delete", label: "删除", danger: true },
];

const groups = computed(() => {
  const set = new Set<string>();
  for (const rule of rules.value) {
    for (const part of String(rule.group || "").split(/[,，;；]/)) {
      if (part.trim()) set.add(part.trim());
    }
  }
  return [...set].sort((a, b) => a.localeCompare(b, "zh-CN"));
});

const groupItems = computed<PopupMenuItem[]>(() => [
  { key: "", label: "全部", checked: !groupFilter.value },
  { key: "none", label: "无分组", checked: groupFilter.value === "none" },
  ...groups.value.map((group) => ({
    key: group,
    label: group,
    checked: groupFilter.value === group,
  })),
]);

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  return rules.value.filter((rule) => {
    if (groupFilter.value === "none" && String(rule.group || "").trim()) return false;
    if (groupFilter.value && groupFilter.value !== "none") {
      if (!String(rule.group || "").includes(groupFilter.value)) return false;
    }
    if (!key) return true;
    return (
      rule.name.toLowerCase().includes(key) ||
      rule.pattern.toLowerCase().includes(key) ||
      String(rule.group || "").toLowerCase().includes(key)
    );
  });
});

const selectedRules = computed(() =>
  filtered.value.filter((rule) => selected.value.has(rule.id)),
);

function displayName(rule: ReplaceRule): string {
  const group = String(rule.group || "").trim();
  return group ? `${rule.name} (${group})` : rule.name;
}

function notify(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2500);
}

async function load() {
  rules.value = await replaceRules();
  const alive = new Set(rules.value.map((rule) => rule.id));
  selected.value = new Set([...selected.value].filter((id) => alive.has(id)));
}

function setSelected(rule: ReplaceRule, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(rule.id);
  else next.delete(rule.id);
  selected.value = next;
}

function selectAll(value: boolean) {
  selected.value = value
    ? new Set(filtered.value.map((rule) => rule.id))
    : new Set();
}

function revertSelection() {
  selected.value = new Set(
    filtered.value.filter((rule) => !selected.value.has(rule.id)).map((rule) => rule.id),
  );
}

function openEditor(id = 0) {
  router.push({ path: "/replace-edit", query: id ? { id: String(id) } : {} });
}

async function toggleEnabled(rule: ReplaceRule) {
  await replaceRuleSave({ ...rule, isEnabled: rule.isEnabled === false });
  await readerInvalidateAll();
  await load();
}

async function setSelectedEnabled(enabled: boolean) {
  for (const rule of selectedRules.value) {
    if ((rule.isEnabled !== false) === enabled) continue;
    await replaceRuleSave({ ...rule, isEnabled: enabled });
  }
  await readerInvalidateAll();
  await load();
}

async function applyOrder(ordered: ReplaceRule[]) {
  for (let index = 0; index < ordered.length; index += 1) {
    if ((ordered[index].sortOrder ?? index) !== index) {
      await replaceRuleSave({ ...ordered[index], sortOrder: index });
    }
  }
  await readerInvalidateAll();
  await load();
}

async function move(rule: ReplaceRule, toTop: boolean) {
  const ordered = [...rules.value].sort(
    (a, b) => (a.sortOrder ?? 0) - (b.sortOrder ?? 0),
  );
  const index = ordered.findIndex((item) => item.id === rule.id);
  if (index < 0) return;
  ordered.splice(index, 1);
  if (toTop) ordered.unshift(rule);
  else ordered.push(rule);
  await applyOrder(ordered);
}

async function moveSelected(toTop: boolean) {
  if (!selectedRules.value.length) return;
  const selectedIds = new Set(selectedRules.value.map((rule) => rule.id));
  const ordered = [...rules.value].sort(
    (a, b) => (a.sortOrder ?? 0) - (b.sortOrder ?? 0),
  );
  const chosen = ordered.filter((rule) => selectedIds.has(rule.id));
  const rest = ordered.filter((rule) => !selectedIds.has(rule.id));
  await applyOrder(toTop ? [...chosen, ...rest] : [...rest, ...chosen]);
}

async function remove(rule: ReplaceRule) {
  if (!confirm(`删除规则「${rule.name}」？`)) return;
  await replaceRuleDelete(rule.id);
  selected.value.delete(rule.id);
  await readerInvalidateAll();
  notify("规则已删除");
  await load();
}

async function deleteSelected() {
  if (!selectedRules.value.length) return;
  if (!confirm(`删除选中的 ${selectedRules.value.length} 条替换规则？`)) return;
  for (const rule of selectedRules.value) await replaceRuleDelete(rule.id);
  selected.value = new Set();
  await readerInvalidateAll();
  notify("规则已删除");
  await load();
}

async function exportSelected() {
  if (!selectedRules.value.length) return;
  const saved = await saveTextFile(
    "replace-rules.json",
    JSON.stringify(selectedRules.value, null, 2),
    "替换规则 JSON",
  );
  if (saved) notify("规则已导出");
}

async function onImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const report = await replaceRuleImport(await file.text());
    await readerInvalidateAll();
    await load();
    notify(`导入完成：新增 ${report.entries.filter((entry) => entry.status === "new").length}`);
  } catch (error) {
    notify(String(error));
  } finally {
    input.value = "";
  }
}

async function importOnline(url: string) {
  try {
    const report = await replaceRuleImport(await fetchRemoteText(url));
    await readerInvalidateAll();
    await load();
    notify(`在线导入完成：共 ${report.entries.length} 条`);
  } catch (error) {
    notify(String(error));
  }
}

async function importQr(text: string) {
  try {
    const report = await replaceRuleImport(text);
    await readerInvalidateAll();
    await load();
    notify(`二维码导入完成：共 ${report.entries.length} 条`);
  } catch (error) {
    notify(String(error));
  }
}

function onMenu(key: string) {
  if (key === "add") openEditor();
  else if (key === "importLocal") importInput.value?.click();
  else if (key === "importOnline") onlineVisible.value = true;
  else if (key === "importQr") qrVisible.value = true;
  else if (key === "help") notify("替换规则帮助暂未接入");
}

function onGroupMenu(key: string) {
  groupFilter.value = key;
}

function openMore(rule: ReplaceRule, event: MouseEvent) {
  moreRule.value = rule;
  moreTop.value = Math.min(event.clientY, window.innerHeight - 180);
  moreRight.value = Math.max(8, window.innerWidth - event.clientX);
  moreVisible.value = true;
}

async function onMore(key: string) {
  const rule = moreRule.value;
  if (!rule) return;
  if (key === "top") await move(rule, true);
  else if (key === "bottom") await move(rule, false);
  else if (key === "delete") await remove(rule);
}

async function onSelectionMenu(key: string) {
  if (key === "enable") await setSelectedEnabled(true);
  else if (key === "disable") await setSelectedEnabled(false);
  else if (key === "top") await moveSelected(true);
  else if (key === "bottom") await moveSelected(false);
  else if (key === "export") await exportSelected();
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
  <div class="replace-page">
    <TitleBar search-only @back="router.back()">
      <template #content>
        <SearchBar v-model="keyword" placeholder="搜索替换规则" />
      </template>
      <template #actions>
        <button aria-label="分组" @click="groupMenuVisible = true">
          <Icon name="ic_groups" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <input
      ref="importInput"
      class="hidden-input"
      type="file"
      accept="application/json,.json"
      @change="onImport"
    />
    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />
    <PopupMenu
      v-model="groupMenuVisible"
      :items="groupItems"
      :top="54"
      @select="onGroupMenu"
    />
    <PopupMenu
      v-model="moreVisible"
      :items="moreItems"
      :top="moreTop"
      :right="moreRight"
      @select="onMore"
    />

    <p v-if="message" class="message">{{ message }}</p>
    <main ref="listRef" class="list">
      <RuleRow
        v-for="rule in filtered"
        :key="rule.id"
        :name="displayName(rule)"
        :enabled="rule.isEnabled !== false"
        :selected="selected.has(rule.id)"
        @update:selected="setSelected(rule, $event)"
        @toggle="toggleEnabled(rule)"
        @edit="openEditor(rule.id)"
        @more="openMore(rule, $event)"
      />
      <p v-if="filtered.length === 0" class="empty">暂无替换规则</p>
    </main>

    <SelectActionBar
      :count="selectedRules.length"
      :total="filtered.length"
      main-text="删除"
      :menu-items="selectionMenuItems"
      @select-all="selectAll"
      @revert="revertSelection"
      @main="deleteSelected"
      @menu="onSelectionMenu"
    />

    <OnlineImportDialog
      v-model="onlineVisible"
      title="在线导入替换规则"
      @confirm="importOnline"
    />
    <QrCodeDialog v-model="qrVisible" mode="import" @import="importQr" />
  </div>
</template>

<style scoped>
.replace-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.hidden-input {
  display: none;
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.empty {
  margin: 40px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
