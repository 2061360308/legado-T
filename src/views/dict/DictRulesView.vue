<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import type { DictRule } from "@/core/domain";
import { fetchRemoteText } from "@/utils/remoteFetch";
import { saveTextFile } from "@/utils/fileSave";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import RuleRow from "@/components/android/RuleRow.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import ImportRuleDialog from "@/components/android/ImportRuleDialog.vue";
import OnlineImportDialog from "@/views/misc/OnlineImportDialog.vue";
import QrCodeDialog from "@/views/source/QrCodeDialog.vue";
import defaultDictRules from "@/assets/defaultData/dictRules.json";
import { parseRuleList, type RuleImportRow } from "@/utils/ruleImport";
import { showError } from "@/utils/errors";
import {
  dictRuleDelete,
  dictRuleImport,
  dictRuleSave,
  dictRules,
} from "@/core/application/rules";

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const rules = ref<DictRule[]>([]);
const editing = ref(false);
const message = ref("");
const form = ref<DictRule>(emptyRule());
const menuVisible = ref(false);
const selected = ref<Set<string>>(new Set());
const importInput = ref<HTMLInputElement | null>(null);
const onlineVisible = ref(false);
const qrVisible = ref(false);
const importVisible = ref(false);
const importRows = ref<RuleImportRow<DictRule>[]>([]);
const importItems = computed(() => importRows.value.map((row) => row.item));

const menuItems: PopupMenuItem[] = [
  { key: "importLocal", label: "本地导入", icon: "ic_import" },
  { key: "importOnline", label: "在线导入", icon: "ic_import" },
  { key: "importQr", label: "二维码导入", icon: "ic_scan" },
  { key: "importDefault", label: "导入默认规则", icon: "ic_import" },
  { key: "help", label: "帮助", icon: "ic_help", muted: true },
];
const selectionMenuItems: PopupMenuItem[] = [
  { key: "enable", label: "启用" },
  { key: "disable", label: "禁用" },
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "export", label: "导出选中" },
];

const selectedRules = computed(() =>
  rules.value.filter((rule) => selected.value.has(rule.name)),
);

function emptyRule(): DictRule {
  return { name: "", urlRule: "", showRule: "", enabled: true, sortNumber: 0, jsLib: "" };
}

function notify(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2500);
}

async function load() {
  rules.value = await dictRules();
  const alive = new Set(rules.value.map((rule) => rule.name));
  selected.value = new Set([...selected.value].filter((name) => alive.has(name)));
}

function startCreate() {
  form.value = emptyRule();
  editing.value = true;
  menuVisible.value = false;
}

function startEdit(rule: DictRule) {
  form.value = { ...rule };
  editing.value = true;
}

async function save() {
  if (!form.value.name.trim() || !form.value.urlRule.trim()) {
    notify("名称与请求规则不能为空");
    return;
  }
  await dictRuleSave({ ...form.value });
  notify("规则已保存");
  editing.value = false;
  await load();
}

async function toggle(rule: DictRule) {
  await dictRuleSave({ ...rule, enabled: rule.enabled === false });
  await load();
}

async function setSelectedEnabled(enabled: boolean) {
  for (const rule of selectedRules.value) {
    if ((rule.enabled !== false) === enabled) continue;
    await dictRuleSave({ ...rule, enabled });
  }
  await load();
}

async function remove(rule: DictRule) {
  if (!confirm(`删除字典规则「${rule.name}」？`)) return;
  await dictRuleDelete(rule.name);
  selected.value.delete(rule.name);
  notify("规则已删除");
  await load();
}

async function deleteSelected() {
  if (!selectedRules.value.length) return;
  if (!confirm(`删除选中的 ${selectedRules.value.length} 条字典规则？`)) return;
  for (const rule of selectedRules.value) await dictRuleDelete(rule.name);
  selected.value = new Set();
  notify("规则已删除");
  await load();
}

async function applyOrder(ordered: DictRule[]) {
  for (let index = 0; index < ordered.length; index += 1) {
    if ((ordered[index].sortNumber ?? index) !== index) {
      await dictRuleSave({ ...ordered[index], sortNumber: index });
    }
  }
  await load();
}

async function moveSelected(toTop: boolean) {
  if (!selectedRules.value.length) return;
  const selectedNames = new Set(selectedRules.value.map((rule) => rule.name));
  const ordered = [...rules.value].sort(
    (a, b) => (a.sortNumber ?? 0) - (b.sortNumber ?? 0),
  );
  const chosen = ordered.filter((rule) => selectedNames.has(rule.name));
  const rest = ordered.filter((rule) => !selectedNames.has(rule.name));
  await applyOrder(toTop ? [...chosen, ...rest] : [...rest, ...chosen]);
}

function setSelected(rule: DictRule, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(rule.name);
  else next.delete(rule.name);
  selected.value = next;
}

function selectAll(value: boolean) {
  selected.value = value ? new Set(rules.value.map((rule) => rule.name)) : new Set();
}

function revertSelection() {
  selected.value = new Set(
    rules.value.filter((rule) => !selected.value.has(rule.name)).map((rule) => rule.name),
  );
}

async function exportSelected() {
  if (!selectedRules.value.length) return;
  const saved = await saveTextFile(
    "dictRules.json",
    JSON.stringify(selectedRules.value, null, 2),
    "字典规则 JSON",
  );
  if (saved) notify("规则已导出");
}

function onMenu(key: string) {
  if (key === "importLocal") importInput.value?.click();
  else if (key === "importOnline") onlineVisible.value = true;
  else if (key === "importQr") qrVisible.value = true;
  else if (key === "importDefault") importDefault();
  else if (key === "help") notify("字典规则帮助暂未接入");
}

async function openImport(json: string) {
  try {
    const values = parseRuleList<DictRule>(json);
    if (!values.length) throw new Error("没有可导入的字典规则");
    const existing = await dictRules();
    const byName = new Map(existing.map((rule) => [rule.name, rule]));
    importRows.value = values.map((value, index) => {
      const old = byName.get(value.name);
      const state = old ? ("existing" as const) : ("new" as const);
      return {
        value,
        item: {
          key: String(index),
          title: value.name || `字典规则 ${index + 1}`,
          subtitle: value.urlRule,
          state,
          selected: state === "new",
        },
      };
    });
    importVisible.value = true;
  } catch (error) {
    showError("导入字典规则", error);
  }
}

async function importSelected(keys: string[]) {
  const values = keys
    .map((key) => importRows.value[Number(key)]?.value)
    .filter((value): value is DictRule => Boolean(value));
  if (!values.length) return;
  try {
    const report = await dictRuleImport(JSON.stringify(values));
    notify(`导入完成：共 ${report.entries.length} 条`);
    await load();
    importRows.value = [];
  } catch (error) {
    showError("导入字典规则", error);
  }
}

async function importDefault() {
  await openImport(JSON.stringify(defaultDictRules));
}

async function onImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    await openImport(await file.text());
  } catch (error) {
    showError("读取字典规则文件", error);
  } finally {
    input.value = "";
  }
}

async function importOnline(url: string) {
  try {
    await openImport(await fetchRemoteText(url));
  } catch (error) {
    showError("在线导入字典规则", error);
  }
}

async function importQr(text: string) {
  await openImport(text);
}

async function onSelectionMenu(key: string) {
  if (key === "enable") await setSelectedEnabled(true);
  else if (key === "disable") await setSelectedEnabled(false);
  else if (key === "top") await moveSelected(true);
  else if (key === "bottom") await moveSelected(false);
  else if (key === "export") await exportSelected();
}

onMounted(load);
</script>

<template>
  <div class="dict-rules">
    <TitleBar title="字典规则" @back="router.back()">
      <template #actions>
        <button aria-label="新增" @click="startCreate">
          <Icon name="ic_add" :size="24" />
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
    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />

    <p v-if="message" class="message">{{ message }}</p>

    <main ref="listRef" class="list">
      <RuleRow
        v-for="rule in rules"
        :key="rule.name"
        :name="rule.name"
        :enabled="rule.enabled !== false"
        :selected="selected.has(rule.name)"
        :show-more="false"
        show-delete
        @update:selected="setSelected(rule, $event)"
        @toggle="toggle(rule)"
        @edit="startEdit(rule)"
        @delete="remove(rule)"
      />
      <p v-if="rules.length === 0" class="empty">暂无字典规则</p>
    </main>

    <SelectActionBar
      :count="selectedRules.length"
      :total="rules.length"
      main-text="删除"
      :menu-items="selectionMenuItems"
      @select-all="selectAll"
      @revert="revertSelection"
      @main="deleteSelected"
      @menu="onSelectionMenu"
    />

    <AndroidDialog
      v-model="editing"
      mode="center"
      :title="form.name ? '编辑字典规则' : '新增字典规则'"
    >
      <div class="form">
        <label>名称<input v-model="form.name" /></label>
        <label>
          请求规则
          <input v-model="form.urlRule" placeholder="https://.../{{key}},{&quot;method&quot;:&quot;GET&quot;}" />
        </label>
        <label>结果规则<input v-model="form.showRule" placeholder="留空直接展示响应" /></label>
        <label>JS 库<input v-model="form.jsLib" /></label>
        <label>排序<input v-model.number="form.sortNumber" type="number" /></label>
        <label class="check">
          <input v-model="form.enabled" type="checkbox" />
          启用
        </label>
      </div>
      <template #footer>
        <button class="btn" @click="editing = false">取消</button>
        <button class="btn primary" @click="save">保存</button>
      </template>
    </AndroidDialog>

    <OnlineImportDialog
      v-model="onlineVisible"
      title="在线导入字典规则"
      @confirm="importOnline"
    />
    <QrCodeDialog v-model="qrVisible" mode="import" @import="importQr" />
    <ImportRuleDialog
      v-model="importVisible"
      title="导入字典规则"
      :items="importItems"
      @confirm="importSelected"
    />
  </div>
</template>

<style scoped>
.dict-rules {
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
  text-align: center;
  color: var(--ac-secondary-text);
  padding: 40px 0;
  font-size: 13px;
}
.form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}
.form label.check {
  flex-direction: row;
  align-items: center;
  gap: 6px;
}
.form input:not([type="checkbox"]) {
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
}
.btn {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 16px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
</style>
