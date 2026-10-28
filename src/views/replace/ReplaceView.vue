<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { ReplaceRule } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import {
  replaceRuleDelete,
  replaceRuleExport,
  replaceRuleImport,
  replaceRuleSave,
  replaceRules,
} from "@/api/rules";

const router = useRouter();
const rules = ref<ReplaceRule[]>([]);
const editing = ref(false);
const message = ref("");
const keyword = ref("");
const groupFilter = ref("");
const form = ref<ReplaceRule>(emptyRule());

function emptyRule(): ReplaceRule {
  return {
    id: 0,
    name: "",
    group: "",
    pattern: "",
    replacement: "",
    scope: "",
    excludeScope: "",
    scopeTitle: false,
    scopeContent: true,
    isEnabled: true,
    isRegex: true,
    timeoutMillisecond: 3000,
    sortOrder: 0,
  };
}

const groups = computed(() => {
  const set = new Set<string>();
  for (const rule of rules.value) {
    for (const part of (rule.group || "").split(/[,，]/)) {
      if (part.trim()) set.add(part.trim());
    }
  }
  return [...set];
});

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  return rules.value.filter((rule) => {
    if (groupFilter.value && !(rule.group || "").includes(groupFilter.value)) return false;
    if (!key) return true;
    return (
      rule.name.toLowerCase().includes(key) ||
      rule.pattern.toLowerCase().includes(key) ||
      (rule.group || "").toLowerCase().includes(key)
    );
  });
});

async function load() {
  rules.value = await replaceRules();
}

function startCreate() {
  form.value = { ...emptyRule(), sortOrder: (rules.value.at(-1)?.sortOrder ?? 0) + 1 };
  editing.value = true;
}

function startEdit(rule: ReplaceRule) {
  form.value = { ...rule };
  editing.value = true;
}

function cancel() {
  editing.value = false;
  form.value = emptyRule();
}

async function save() {
  if (!form.value.name.trim() || !form.value.pattern) {
    message.value = "规则名与替换内容不能为空";
    return;
  }
  try {
    await replaceRuleSave({ ...form.value });
    message.value = "规则已保存";
    cancel();
    await load();
  } catch (e) {
    message.value = String(e);
  }
}

async function toggle(rule: ReplaceRule) {
  await replaceRuleSave({ ...rule, isEnabled: !rule.isEnabled });
  await load();
}

async function remove(rule: ReplaceRule) {
  if (!confirm(`删除替换规则「${rule.name}」？`)) return;
  await replaceRuleDelete(rule.id);
  message.value = "规则已删除";
  await load();
}

async function move(index: number, delta: number) {
  const target = index + delta;
  if (target < 0 || target >= rules.value.length) return;
  const a = rules.value[index];
  const b = rules.value[target];
  const orderA = a.sortOrder ?? index;
  const orderB = b.sortOrder ?? target;
  await replaceRuleSave({ ...a, sortOrder: orderB });
  await replaceRuleSave({ ...b, sortOrder: orderA });
  await load();
}

async function onImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const report = await replaceRuleImport(await file.text());
    const added = report.entries.filter((e) => e.status === "new").length;
    const updated = report.entries.filter((e) => e.status === "updated").length;
    message.value = `导入完成：新增 ${added}，更新 ${updated}`;
    await load();
  } catch (e) {
    message.value = String(e);
  } finally {
    input.value = "";
  }
}

async function onExport() {
  const json = await replaceRuleExport();
  download("replaceRule.json", json);
}

function download(name: string, content: string) {
  const blob = new Blob([content], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  URL.revokeObjectURL(url);
}

onMounted(load);
</script>

<template>
  <div class="replace-view">
    <TitleBar title="替换净化" @back="router.back()" />
    <header class="toolbar">
      <input v-model="keyword" class="search" placeholder="搜索名称/正则/分组" />
      <select v-model="groupFilter">
        <option value="">全部分组</option>
        <option v-for="g in groups" :key="g" :value="g">{{ g }}</option>
      </select>
      <button @click="startCreate">新增</button>
      <label class="file-btn">
        导入
        <input type="file" accept="application/json,.json" @change="onImport" />
      </label>
      <button @click="onExport">导出</button>
    </header>

    <p v-if="message" class="message">{{ message }}</p>

    <form v-if="editing" class="editor" @submit.prevent="save">
      <h2>{{ form.id ? "编辑替换规则" : "新增替换规则" }}</h2>
      <div class="grid">
        <label>名称<input v-model="form.name" /></label>
        <label>分组<input v-model="form.group" placeholder="逗号分隔" /></label>
      </div>
      <label>替换内容<input v-model="form.pattern" placeholder="正则或普通字符串" /></label>
      <label>替换为<input v-model="form.replacement" placeholder="留空表示删除" /></label>
      <div class="grid">
        <label>作用范围<input v-model="form.scope" placeholder="书名/书源名子串，逗号分隔" /></label>
        <label>排除范围<input v-model="form.excludeScope" /></label>
      </div>
      <div class="grid">
        <label>超时毫秒<input v-model.number="form.timeoutMillisecond" type="number" /></label>
        <label>排序<input v-model.number="form.sortOrder" type="number" /></label>
      </div>
      <div class="checks">
        <label><input v-model="form.isEnabled" type="checkbox" />启用</label>
        <label><input v-model="form.isRegex" type="checkbox" />正则</label>
        <label><input v-model="form.scopeTitle" type="checkbox" />作用于标题</label>
        <label><input v-model="form.scopeContent" type="checkbox" />作用于正文</label>
      </div>
      <div class="editor-actions">
        <button type="submit">保存</button>
        <button type="button" @click="cancel">取消</button>
      </div>
    </form>

    <ol class="rule-list">
      <li v-for="(rule, index) in filtered" :key="rule.id">
        <label class="check">
          <input type="checkbox" :checked="rule.isEnabled" @change="toggle(rule)" />
        </label>
        <div class="rule-text">
          <p class="rule-name">
            {{ rule.name }}
            <span v-if="rule.group" class="tag">{{ rule.group }}</span>
            <span class="tag">{{ rule.isRegex ? "正则" : "文本" }}</span>
            <span v-if="rule.scopeTitle" class="tag">标题</span>
            <span v-if="rule.scopeContent" class="tag">正文</span>
          </p>
          <code>{{ rule.pattern }} → {{ rule.replacement || "（空）" }}</code>
          <p v-if="rule.scope || rule.excludeScope" class="rule-example">
            范围：{{ rule.scope || "全部" }}
            <span v-if="rule.excludeScope"> · 排除：{{ rule.excludeScope }}</span>
          </p>
        </div>
        <div class="rule-actions">
          <button :disabled="index === 0" @click="move(index, -1)">上移</button>
          <button :disabled="index === filtered.length - 1" @click="move(index, 1)">下移</button>
          <button @click="startEdit(rule)">编辑</button>
          <button class="danger" @click="remove(rule)">删除</button>
        </div>
      </li>
    </ol>
    <p v-if="filtered.length === 0" class="empty">暂无替换规则</p>
  </div>
</template>

<style scoped>
.replace-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  background: var(--ac-background);
}
.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  flex-wrap: wrap;
}
.toolbar h1 {
  font-size: 18px;
  margin: 0 auto 0 0;
}
.search {
  padding: 6px 10px;
  border: 1px solid #ccc;
  border-radius: 8px;
  min-width: 160px;
}
.file-btn {
  position: relative;
  overflow: hidden;
  border: 1px solid #ccc;
  background: #fff;
  border-radius: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 14px;
}
.file-btn input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}
.message {
  color: var(--lr-color-primary);
  font-size: 13px;
}
.editor {
  margin: 12px 0;
  padding: 12px;
  border: 1px solid #e2dccf;
  border-radius: 10px;
  background: #fffdf8;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.editor h2 {
  margin: 0;
  font-size: 16px;
}
.editor label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}
.editor input:not([type="checkbox"]),
.editor select {
  padding: 7px;
  border: 1px solid #ccc;
  border-radius: 8px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.checks {
  display: flex;
  gap: 14px;
  flex-wrap: wrap;
}
.checks label {
  flex-direction: row;
  align-items: center;
  gap: 4px;
}
.editor-actions {
  display: flex;
  gap: 8px;
}
.rule-list {
  list-style: none;
  margin: 12px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.rule-list li {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px;
  border: 1px solid #eee7da;
  border-radius: 10px;
  background: var(--lr-color-surface);
}
.rule-text {
  flex: 1;
  min-width: 0;
}
.rule-name {
  margin: 0;
  font-weight: 600;
  display: flex;
  gap: 6px;
  align-items: center;
  flex-wrap: wrap;
}
.tag {
  font-size: 11px;
  color: #6b6154;
  background: #efe8db;
  border-radius: 4px;
  padding: 1px 5px;
  font-weight: normal;
}
.rule-text code {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  word-break: break-all;
  color: #6b6154;
}
.rule-example {
  margin: 4px 0 0;
  font-size: 12px;
  color: #8a8175;
}
.rule-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.danger {
  color: #c62828;
}
button {
  border: 1px solid #d8d0c0;
  background: #fffdf8;
  border-radius: 8px;
  padding: 6px 10px;
  cursor: pointer;
  font-size: 13px;
}
button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.empty {
  color: #999;
  text-align: center;
  padding: 24px 0;
}
</style>
