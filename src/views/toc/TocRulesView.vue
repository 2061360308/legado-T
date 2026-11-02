<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import type { TxtTocRule } from "@/core/domain";
import {
  txtTocRuleDelete,
  txtTocRuleImport,
  txtTocRuleReorder,
  txtTocRuleRestoreDefault,
  txtTocRuleSave,
  txtTocRules,
} from "@/core/application/rules";
import { fetchRemoteText } from "@/utils/remoteFetch";
import { saveTextFile } from "@/utils/fileSave";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import RuleRow from "@/components/android/RuleRow.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import OnlineImportDialog from "@/views/misc/OnlineImportDialog.vue";
import QrCodeDialog from "@/views/source/QrCodeDialog.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const menuItems: PopupMenuItem[] = [
  { key: "import", label: "导入本地", icon: "ic_import" },
  { key: "importOnline", label: "在线导入", icon: "ic_import" },
  { key: "importQr", label: "二维码导入", icon: "ic_scan" },
  { key: "restore", label: "恢复默认规则", icon: "ic_import" },
  { key: "help", label: "帮助", icon: "ic_help", muted: true },
];
const selectionMenuItems: PopupMenuItem[] = [
  { key: "enable", label: "启用" },
  { key: "disable", label: "禁用" },
  { key: "export", label: "导出选中" },
];
const ruleMenuItems: PopupMenuItem[] = [
  { key: "top", label: "置顶" },
  { key: "bottom", label: "置底" },
  { key: "delete", label: "删除", danger: true },
];

const rules = ref<TxtTocRule[]>([]);
const selected = ref(new Set<number>());
const editing = ref(false);
const menuVisible = ref(false);
const moreVisible = ref(false);
const moreRule = ref<TxtTocRule | null>(null);
const moreTop = ref(52);
const moreRight = ref(8);
const importInput = ref<HTMLInputElement | null>(null);
const onlineVisible = ref(false);
const qrVisible = ref(false);
const message = ref("");
const form = ref<TxtTocRule>(emptyRule());

function emptyRule(): TxtTocRule {
  return { id: 0, name: "", rule: "", example: "", serialNumber: 0, enable: true };
}

function notify(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2500);
}

const selectedRules = computed(() =>
  rules.value.filter((rule) => selected.value.has(rule.id)),
);

async function load() {
  rules.value = await txtTocRules();
  const alive = new Set(rules.value.map((rule) => rule.id));
  selected.value = new Set([...selected.value].filter((id) => alive.has(id)));
}

function startCreate() {
  form.value = emptyRule();
  editing.value = true;
  menuVisible.value = false;
}

function startEdit(rule: TxtTocRule) {
  form.value = { ...rule };
  editing.value = true;
}

function setSelected(rule: TxtTocRule, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(rule.id);
  else next.delete(rule.id);
  selected.value = next;
}

function selectAll(value: boolean) {
  selected.value = value ? new Set(rules.value.map((rule) => rule.id)) : new Set();
}

function revertSelection() {
  selected.value = new Set(
    rules.value.filter((rule) => !selected.value.has(rule.id)).map((rule) => rule.id),
  );
}

async function save() {
  if (!form.value.name.trim() || !form.value.rule.trim()) {
    notify("规则名与正则不能为空");
    return;
  }
  try {
    await txtTocRuleSave({ ...form.value });
    notify("规则已保存");
    editing.value = false;
    await load();
  } catch (error) {
    notify(String(error));
  }
}

async function toggle(rule: TxtTocRule) {
  await txtTocRuleSave({ ...rule, enable: rule.enable === false });
  await load();
}

async function setSelectedEnabled(enabled: boolean) {
  for (const rule of selectedRules.value) {
    if ((rule.enable !== false) === enabled) continue;
    await txtTocRuleSave({ ...rule, enable: enabled });
  }
  await load();
}

async function remove(rule: TxtTocRule) {
  if (!confirm(`删除规则「${rule.name}」？`)) return;
  await txtTocRuleDelete(rule.id);
  selected.value.delete(rule.id);
  notify("规则已删除");
  await load();
}

async function deleteSelected() {
  if (!selectedRules.value.length) return;
  if (!confirm(`删除选中的 ${selectedRules.value.length} 条目录规则？`)) return;
  for (const rule of selectedRules.value) await txtTocRuleDelete(rule.id);
  selected.value = new Set();
  notify("规则已删除");
  await load();
}

async function reorder(ids: number[]) {
  await txtTocRuleReorder(ids);
  await load();
}

async function move(rule: TxtTocRule, toTop: boolean) {
  const ids = rules.value.map((item) => item.id);
  const index = ids.indexOf(rule.id);
  if (index < 0) return;
  ids.splice(index, 1);
  if (toTop) ids.unshift(rule.id);
  else ids.push(rule.id);
  await reorder(ids);
}

async function moveSelected(toTop: boolean) {
  if (!selectedRules.value.length) return;
  const selectedIds = new Set(selectedRules.value.map((rule) => rule.id));
  const chosen = rules.value.filter((rule) => selectedIds.has(rule.id));
  const rest = rules.value.filter((rule) => !selectedIds.has(rule.id));
  const next = toTop ? [...chosen, ...rest] : [...rest, ...chosen];
  await reorder(next.map((rule) => rule.id));
}

async function exportSelected() {
  if (!selectedRules.value.length) return;
  const saved = await saveTextFile(
    "txtTocRule.json",
    JSON.stringify(selectedRules.value, null, 2),
    "目录规则 JSON",
  );
  if (saved) notify("规则已导出");
}

async function restoreDefault() {
  menuVisible.value = false;
  if (!confirm("恢复默认目录规则？当前自定义规则将被覆盖。")) return;
  const count = await txtTocRuleRestoreDefault();
  notify(`已恢复 ${count} 条默认规则`);
  await load();
}

async function onImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  menuVisible.value = false;
  try {
    const report = await txtTocRuleImport(await file.text());
    const added = report.entries.filter((entry) => entry.status === "new").length;
    const updated = report.entries.filter((entry) => entry.status === "updated").length;
    notify(`导入完成：新增 ${added}，更新 ${updated}`);
    await load();
  } catch (error) {
    notify(String(error));
  } finally {
    input.value = "";
  }
}

async function importOnline(url: string) {
  try {
    const report = await txtTocRuleImport(await fetchRemoteText(url));
    notify(`在线导入完成：共 ${report.entries.length} 条`);
    await load();
  } catch (error) {
    notify(String(error));
  }
}

async function importQr(text: string) {
  try {
    const report = await txtTocRuleImport(text);
    notify(`二维码导入完成：共 ${report.entries.length} 条`);
    await load();
  } catch (error) {
    notify(String(error));
  }
}

function onMenuSelect(key: string) {
  if (key === "import") importInput.value?.click();
  else if (key === "importOnline") onlineVisible.value = true;
  else if (key === "importQr") qrVisible.value = true;
  else if (key === "restore") restoreDefault();
  else if (key === "help") notify("帮助暂未接入");
}

function openMore(rule: TxtTocRule, event: MouseEvent) {
  moreRule.value = rule;
  moreTop.value = Math.min(event.clientY, window.innerHeight - 180);
  moreRight.value = Math.max(8, window.innerWidth - event.clientX);
  moreVisible.value = true;
}

async function onRuleMenu(key: string) {
  const rule = moreRule.value;
  if (!rule) return;
  if (key === "top") await move(rule, true);
  else if (key === "bottom") await move(rule, false);
  else if (key === "delete") await remove(rule);
}

async function onSelectionMenu(key: string) {
  if (key === "enable") await setSelectedEnabled(true);
  else if (key === "disable") await setSelectedEnabled(false);
  else if (key === "export") await exportSelected();
}

onMounted(load);
</script>

<template>
  <div class="toc-rules">
    <TitleBar title="目录规则" @back="router.back()">
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
    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenuSelect" />
    <PopupMenu
      v-model="moreVisible"
      :items="ruleMenuItems"
      :top="moreTop"
      :right="moreRight"
      @select="onRuleMenu"
    />

    <p v-if="message" class="message">{{ message }}</p>

    <main ref="listRef" class="rule-list">
      <RuleRow
        v-for="rule in rules"
        :key="rule.id"
        :name="rule.name"
        :subtitle="rule.example || ''"
        :enabled="rule.enable !== false"
        :selected="selected.has(rule.id)"
        @update:selected="setSelected(rule, $event)"
        @toggle="toggle(rule)"
        @edit="startEdit(rule)"
        @more="openMore(rule, $event)"
      />
      <p v-if="rules.length === 0" class="empty">暂无目录规则</p>
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
      :title="form.id ? '编辑目录规则' : '新增目录规则'"
    >
      <div class="form">
        <label>名称<input v-model="form.name" placeholder="如：第x章" /></label>
        <label>正则<input v-model="form.rule" placeholder="^第[一二三四五六七八九十百千0-9]+章.*$" /></label>
        <label>示例<input v-model="form.example" placeholder="第一章 起点" /></label>
        <label class="check">
          <input v-model="form.enable" type="checkbox" />
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
      title="在线导入目录规则"
      @confirm="importOnline"
    />
    <QrCodeDialog v-model="qrVisible" mode="import" @import="importQr" />
  </div>
</template>

<style scoped>
.toc-rules {
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
.rule-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
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
.empty {
  text-align: center;
  color: var(--ac-secondary-text);
  padding: 40px 0;
  font-size: 13px;
}
</style>
