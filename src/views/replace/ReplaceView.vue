<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { ReplaceRule } from "@/core/domain";
import {
  replaceRuleDelete,
  replaceRuleExport,
  replaceRuleImport,
  replaceRuleSave,
  replaceRules,
} from "@/core/application/rules";
import { readerInvalidateAll } from "@/core/application/reader";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const rules = ref<ReplaceRule[]>([]);
const keyword = ref("");
const groupFilter = ref("");
const selected = ref(new Set<number>());
const message = ref("");
const menuVisible = ref(false);
const groupMenuVisible = ref(false);
const moreVisible = ref(false);

const groups = computed(() => {
  const set = new Set<string>();
  for (const rule of rules.value) {
    for (const part of (rule.group || "").split(/[,，]/)) {
      if (part.trim()) set.add(part.trim());
    }
  }
  return [...set];
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

const menuItems: PopupMenuItem[] = [
  { key: "add", label: "添加替换规则" },
  { key: "import", label: "本地导入" },
  { key: "help", label: "帮助" },
];

const moreItems: PopupMenuItem[] = [
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "delete", label: "删除", danger: true },
];

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  return rules.value.filter((rule) => {
    if (groupFilter.value === "none" && (rule.group || "").trim()) return false;
    if (groupFilter.value && groupFilter.value !== "none") {
      if (!(rule.group || "").includes(groupFilter.value)) return false;
    }
    if (!key) return true;
    return (
      rule.name.toLowerCase().includes(key) ||
      rule.pattern.toLowerCase().includes(key) ||
      (rule.group || "").toLowerCase().includes(key)
    );
  });
});

const allChecked = computed(
  () => filtered.value.length > 0 && filtered.value.every((rule) => selected.value.has(rule.id)),
);

async function load() {
  rules.value = await replaceRules();
  selected.value = new Set();
}

function toggle(rule: ReplaceRule, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(rule.id);
  else next.delete(rule.id);
  selected.value = next;
}

function toggleAll() {
  if (allChecked.value) selected.value = new Set();
  else selected.value = new Set(filtered.value.map((rule) => rule.id));
}

function revertSelection() {
  const next = new Set<number>();
  for (const rule of filtered.value) {
    if (!selected.value.has(rule.id)) next.add(rule.id);
  }
  selected.value = next;
}

function openEditor(id = 0) {
  router.push({ path: "/replace-edit", query: id ? { id: String(id) } : {} });
}

async function toggleEnabled(rule: ReplaceRule) {
  await replaceRuleSave({ ...rule, isEnabled: !rule.isEnabled });
  await readerInvalidateAll();
  await load();
}

async function moveSelected(toTop: boolean) {
  const selectedRules = rules.value.filter((rule) => selected.value.has(rule.id));
  if (!selectedRules.length) return;
  const rest = rules.value.filter((rule) => !selected.value.has(rule.id));
  const ordered = toTop ? [...selectedRules, ...rest] : [...rest, ...selectedRules];
  for (let index = 0; index < ordered.length; index += 1) {
    if ((ordered[index].sortOrder ?? index) !== index) {
      await replaceRuleSave({ ...ordered[index], sortOrder: index });
    }
  }
  await readerInvalidateAll();
  await load();
}

async function removeSelected() {
  if (!selected.value.size) return;
  if (!confirm(`删除选中的 ${selected.value.size} 条替换规则？`)) return;
  for (const id of selected.value) await replaceRuleDelete(id);
  await readerInvalidateAll();
  await load();
}

async function onMenu(key: string) {
  if (key === "add") openEditor();
  else if (key === "import") {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,application/json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const report = await replaceRuleImport(await file.text());
        await readerInvalidateAll();
        await load();
        message.value = `导入完成：新增 ${report.entries.filter((e) => e.status === "new").length}`;
      } catch (e) {
        message.value = String(e);
      }
    };
    input.click();
  } else if (key === "help") {
    message.value = "替换规则帮助暂未接入";
  }
}

function onGroupMenu(key: string) {
  groupFilter.value = key;
}

async function onMore(key: string) {
  if (key === "top") await moveSelected(true);
  else if (key === "bottom") await moveSelected(false);
  else if (key === "delete") await removeSelected();
}

async function exportRules() {
  const json = await replaceRuleExport();
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "replace-rules.json";
  anchor.click();
  URL.revokeObjectURL(url);
  message.value = "规则已导出";
}

onMounted(load);
</script>

<template>
  <div class="replace-page">
    <TitleBar title="替换净化" @back="router.back()">
      <template #content>
        <label class="search-box">
          <Icon name="ic_search_hint" :size="18" />
          <input v-model="keyword" placeholder="搜索规则" />
        </label>
      </template>
      <template #actions>
        <button aria-label="分组" @click="groupMenuVisible = true">
          <Icon name="ic_groups" :size="24" />
        </button>
        <button aria-label="添加" @click="openEditor()">
          <Icon name="ic_add" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>
    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />
    <PopupMenu v-model="groupMenuVisible" :items="groupItems" :top="54" @select="onGroupMenu" />
    <PopupMenu v-model="moreVisible" :items="moreItems" :top="52" :right="10" @select="onMore" />

    <p v-if="message" class="message">{{ message }}</p>
    <main class="list">
      <div v-for="rule in filtered" :key="rule.id" class="rule-row">
        <input
          type="checkbox"
          class="check"
          :checked="selected.has(rule.id)"
          @change="toggle(rule, ($event.target as HTMLInputElement).checked)"
        />
        <span class="name">{{ rule.name }}</span>
        <label class="switch">
          <input type="checkbox" :checked="rule.isEnabled !== false" @change="toggleEnabled(rule)" />
        </label>
        <button class="icon-button" aria-label="编辑" @click="openEditor(rule.id)">
          <Icon name="ic_edit" :size="20" />
        </button>
        <button class="icon-button" aria-label="更多" @click="selected = new Set([rule.id]); moreVisible = true">
          <Icon name="ic_more_vert" :size="20" />
        </button>
      </div>
      <p v-if="filtered.length === 0" class="empty">暂无替换规则</p>
    </main>

    <div class="select-bar">
      <label class="select-all" @click.prevent="toggleAll">
        <input type="checkbox" :checked="allChecked" readonly />
        <span>{{ allChecked ? "取消全选" : "全选" }}（{{ selected.size }}/{{ filtered.length }}）</span>
      </label>
      <button :disabled="!selected.size" @click="revertSelection">反选</button>
      <button :disabled="!selected.size" @click="moreVisible = true">更多</button>
      <button :disabled="!selected.size" @click="exportRules">导出</button>
    </div>
  </div>
</template>

<style scoped>
.replace-page {
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
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  overflow-y: auto;
}
.rule-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.check {
  flex: none;
  accent-color: var(--ac-primary);
}
.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
}
.switch input {
  width: 34px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.icon-button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.select-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 8px 6px 16px;
  background: var(--ac-background-menu);
  box-shadow: 0 -2px 2px rgba(0, 0, 0, 0.18);
}
.select-all {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
.select-bar button {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  min-width: 60px;
  padding: 6px 8px;
  font-size: 13px;
}
.select-bar button:disabled {
  opacity: 0.35;
}
.empty {
  margin: 40px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
