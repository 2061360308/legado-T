<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { DictRule } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import RuleRow from "@/components/android/RuleRow.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import { dictRuleDelete, dictRuleExport, dictRuleImport, dictRuleSave, dictRules } from "@/core/application/rules";

const router = useRouter();
const rules = ref<DictRule[]>([]);
const editing = ref(false);
const message = ref("");
const form = ref<DictRule>(emptyRule());
const menuVisible = ref(false);
const moreRule = ref<DictRule | null>(null);
const selected = ref<Set<string>>(new Set());
const importInput = ref<HTMLInputElement | null>(null);

const menuItems: PopupMenuItem[] = [
  { key: "import", label: "导入规则", icon: "ic_import" },
  { key: "export", label: "导出规则", icon: "ic_export" },
];
const ruleMenuItems: PopupMenuItem[] = [
  { key: "up", label: "上移" },
  { key: "down", label: "下移" },
  { key: "delete", label: "删除", danger: true },
];

function emptyRule(): DictRule {
  return { name: "", urlRule: "", showRule: "", enabled: true, sortNumber: 0, jsLib: "" };
}

async function load() {
  rules.value = await dictRules();
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
    message.value = "名称与请求规则不能为空";
    return;
  }
  await dictRuleSave({ ...form.value });
  message.value = "规则已保存";
  editing.value = false;
  await load();
}

async function toggle(rule: DictRule) {
  await dictRuleSave({ ...rule, enabled: !rule.enabled });
  await load();
}

async function remove(rule: DictRule) {
  if (!confirm(`删除字典规则「${rule.name}」？`)) return;
  await dictRuleDelete(rule.name);
  message.value = "规则已删除";
  selected.value.delete(rule.name);
  await load();
}

function select(rule: DictRule, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(rule.name);
  else next.delete(rule.name);
  selected.value = next;
}

function cancel() {
  selected.value = new Set();
}

async function batchEnable(enabled: boolean) {
  for (const name of selected.value) {
    const rule = rules.value.find((item) => item.name === name);
    if (rule) await dictRuleSave({ ...rule, enabled });
  }
  cancel();
  await load();
}

async function batchDelete() {
  for (const name of selected.value) await dictRuleDelete(name);
  cancel();
  await load();
}

async function move(rule: DictRule, delta: number) {
  const index = rules.value.findIndex((item) => item.name === rule.name);
  const target = index + delta;
  if (index < 0 || target < 0 || target >= rules.value.length) return;
  const a = rules.value[index];
  const b = rules.value[target];
  await dictRuleSave({ ...a, sortNumber: b.sortNumber ?? target });
  await dictRuleSave({ ...b, sortNumber: a.sortNumber ?? index });
  await load();
}

function onRuleMenu(key: string) {
  const rule = moreRule.value;
  if (!rule) return;
  if (key === "up") move(rule, -1);
  else if (key === "down") move(rule, 1);
  else if (key === "delete") remove(rule);
}

function onMenu(key: string) {
  if (key === "import") importInput.value?.click();
  else if (key === "export") onExport();
}

async function onImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const report = await dictRuleImport(await file.text());
    message.value = `导入完成：共 ${report.entries.length} 条`;
    await load();
  } catch (e) {
    message.value = String(e);
  } finally {
    input.value = "";
  }
}

async function onExport() {
  const json = await dictRuleExport();
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "dictRules.json";
  a.click();
  URL.revokeObjectURL(url);
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
    <PopupMenu
      :model-value="!!moreRule"
      :items="ruleMenuItems"
      @update:model-value="moreRule = null"
      @select="onRuleMenu"
    />

    <p v-if="message" class="message">{{ message }}</p>

    <main class="list">
      <RuleRow
        v-for="rule in rules"
        :key="rule.name"
        :name="rule.name"
        :subtitle="rule.urlRule"
        :enabled="rule.enabled !== false"
        :selected="selected.has(rule.name)"
        @update:selected="select(rule, $event)"
        @toggle="toggle(rule)"
        @edit="startEdit(rule)"
        @more="moreRule = rule"
      />
      <p v-if="rules.length === 0" class="empty">暂无字典规则</p>
    </main>

    <SelectActionBar
      :count="selected.size"
      @enable="batchEnable(true)"
      @disable="batchEnable(false)"
      @remove="batchDelete"
      @cancel="cancel"
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
  </div>
</template>

<style scoped>
.dict-rules {
  height: 100%;
  display: flex;
  flex-direction: column;
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
