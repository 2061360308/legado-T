<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { TxtTocRule } from "@/api/bindings";
import {
  txtTocRuleDelete,
  txtTocRuleExport,
  txtTocRuleImport,
  txtTocRuleReorder,
  txtTocRuleRestoreDefault,
  txtTocRuleSave,
  txtTocRules,
} from "@/api/rules";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const menuItems: PopupMenuItem[] = [
  { key: "import", label: "导入本地", icon: "ic_import" },
  { key: "export", label: "导出规则", icon: "ic_export" },
  { key: "restore", label: "恢复默认规则", icon: "ic_import" },
  { key: "help", label: "帮助", icon: "ic_help", muted: true },
];
const rules = ref<TxtTocRule[]>([]);
const editing = ref(false);
const menuVisible = ref(false);
const ruleMenu = ref<{ rule: TxtTocRule; index: number } | null>(null);
const importInput = ref<HTMLInputElement | null>(null);
const message = ref("");
const form = ref<TxtTocRule>(emptyRule());

function emptyRule(): TxtTocRule {
  return { id: 0, name: "", rule: "", example: "", serialNumber: 0, enable: true };
}

function notify(text: string) {
  message.value = text;
}

async function load() {
  rules.value = await txtTocRules();
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

function showRuleMenu(rule: TxtTocRule, index: number) {
  ruleMenu.value = { rule, index };
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
  } catch (e) {
    notify(String(e));
  }
}

async function toggle(rule: TxtTocRule) {
  await txtTocRuleSave({ ...rule, enable: !rule.enable });
  await load();
}

async function remove(rule: TxtTocRule) {
  if (!confirm(`删除规则「${rule.name}」？`)) return;
  await txtTocRuleDelete(rule.id);
  notify("规则已删除");
  await load();
}

async function move(index: number, delta: number) {
  const target = index + delta;
  if (target < 0 || target >= rules.value.length) return;
  const ids = rules.value.map((r) => r.id);
  [ids[index], ids[target]] = [ids[target], ids[index]];
  await txtTocRuleReorder(ids);
  await load();
}

function onMenuSelect(key: string) {
  if (key === "import") importInput.value?.click();
  else if (key === "export") onExport();
  else if (key === "restore") restoreDefault();
  else if (key === "help") notify("帮助暂未实现");
}

async function restoreDefault() {
  menuVisible.value = false;
  if (!confirm("恢复默认 26 条目录规则？当前自定义规则将被覆盖。")) return;
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
    const added = report.entries.filter((e) => e.status === "new").length;
    const updated = report.entries.filter((e) => e.status === "updated").length;
    notify(`导入完成：新增 ${added}，更新 ${updated}`);
    await load();
  } catch (e) {
    notify(String(e));
  } finally {
    input.value = "";
  }
}

async function onExport() {
  menuVisible.value = false;
  const json = await txtTocRuleExport();
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "txtTocRule.json";
  a.click();
  URL.revokeObjectURL(url);
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
        <button aria-label="更多" @click="menuVisible = !menuVisible">
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

    <p v-if="message" class="message">{{ message }}</p>

    <main class="rule-list">
      <div v-for="(rule, index) in rules" :key="rule.id" class="rule-row">
        <label class="row-check">
          <input type="checkbox" :checked="rule.enable" @change="toggle(rule)" />
        </label>
        <div class="rule-text">
          <p class="rule-name">{{ rule.name }}</p>
          <p class="rule-example">{{ rule.example || rule.rule }}</p>
        </div>
        <button class="icon-btn" aria-label="编辑" @click="startEdit(rule)">
          <Icon name="ic_edit" :size="22" />
        </button>
        <button class="icon-btn" aria-label="更多" @click="menuVisible = false; showRuleMenu(rule, index)">
          <Icon name="ic_more_vert" :size="22" />
        </button>
      </div>
      <p v-if="rules.length === 0" class="empty">暂无目录规则</p>
    </main>

    <div v-if="ruleMenu" class="popup-mask" @click.self="ruleMenu = null">
      <div class="popup-menu">
        <button @click="move(ruleMenu.index, -1); ruleMenu = null">上移</button>
        <button @click="move(ruleMenu.index, 1); ruleMenu = null">下移</button>
        <button class="danger" @click="remove(ruleMenu.rule); ruleMenu = null">删除</button>
        <button @click="ruleMenu = null">取消</button>
      </div>
    </div>

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
  </div>
</template>

<style scoped>
.toc-rules {
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
.rule-list {
  flex: 1;
  overflow-y: auto;
}
.rule-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.rule-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.rule-name {
  margin: 0;
  font-size: 15px;
  color: var(--ac-primary-text);
}
.rule-example {
  margin: 0;
  font-size: 12px;
  color: var(--ac-secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.row-check input {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.icon-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 50%;
}
.icon-btn:hover {
  background: var(--ac-transparent-10);
}
.popup-mask {
  position: fixed;
  inset: 0;
  z-index: 50;
}
.popup-menu {
  position: absolute;
  top: 52px;
  right: 8px;
  min-width: 180px;
  background: var(--ac-background);
  border-radius: 2px;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.24);
  display: flex;
  flex-direction: column;
  padding: 4px 0;
}
.popup-menu button,
.popup-menu label {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
}
.popup-menu button:hover,
.popup-menu label:hover {
  background: var(--ac-transparent-10);
}
.popup-menu label {
  position: relative;
  overflow: hidden;
}
.popup-menu label input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}
.popup-menu .danger {
  color: var(--ac-error);
}
.popup-menu .muted {
  color: var(--ac-secondary-text);
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
