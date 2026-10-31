<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import type { HttpTts } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import {
  httpTtsDelete,
  httpTtsEngines,
  httpTtsExport,
  httpTtsImport,
  httpTtsSave,
} from "@/core/application/rules";

const router = useRouter();
const engines = ref<HttpTts[]>([]);
const editing = ref(false);
const message = ref("");
const form = ref<HttpTts>(emptyEngine());

function emptyEngine(): HttpTts {
  return {
    id: 0,
    name: "",
    url: "",
    contentType: "audio/mpeg",
    concurrentRate: "0",
    loginUrl: "",
    loginUi: "",
    header: "",
    jsLib: "",
    enabledCookieJar: false,
    loginCheckJs: "",
    lastUpdateTime: 0,
  };
}

async function load() {
  engines.value = await httpTtsEngines();
}

function startCreate() {
  form.value = emptyEngine();
  editing.value = true;
}

function startEdit(engine: HttpTts) {
  form.value = { ...engine };
  editing.value = true;
}

async function save() {
  if (!form.value.name.trim() || !form.value.url.trim()) {
    message.value = "名称与合成 URL 不能为空";
    return;
  }
  await httpTtsSave({ ...form.value });
  message.value = "语音引擎已保存";
  editing.value = false;
  await load();
}

async function remove(engine: HttpTts) {
  if (!confirm(`删除语音引擎「${engine.name}」？`)) return;
  await httpTtsDelete(engine.id);
  message.value = "语音引擎已删除";
  await load();
}

async function onImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const report = await httpTtsImport(await file.text());
    message.value = `导入完成：共 ${report.entries.length} 条`;
    await load();
  } catch (e) {
    message.value = String(e);
  } finally {
    input.value = "";
  }
}

async function onExport() {
  const json = await httpTtsExport();
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "httpTTS.json";
  a.click();
  URL.revokeObjectURL(url);
}

onMounted(load);
</script>

<template>
  <div class="aloud-view">
    <TitleBar title="语音引擎" @back="router.back()" />
    <header class="toolbar">
      <button @click="startCreate">新增在线引擎</button>
      <label class="file-btn">
        导入
        <input type="file" accept="application/json,.json" @change="onImport" />
      </label>
      <button @click="onExport">导出</button>
    </header>

    <p class="hint">系统 TTS 始终可用；在线引擎在阅读页「朗读」面板中选择。</p>
    <p v-if="message" class="message">{{ message }}</p>

    <form v-if="editing" class="editor" @submit.prevent="save">
      <h2>{{ form.id ? "编辑在线语音引擎" : "新增在线语音引擎" }}</h2>
      <div class="grid">
        <label>名称<input v-model="form.name" /></label>
        <label>Content-Type<input v-model="form.contentType" /></label>
      </div>
      <label>合成 URL<input v-model="form.url" placeholder="https://...?text={{speakText}}&speed={{speakSpeed}}" /></label>
      <div class="grid">
        <label>并发率<input v-model="form.concurrentRate" placeholder="0 表示不限" /></label>
        <label>登录地址<input v-model="form.loginUrl" /></label>
      </div>
      <label>请求头<input v-model="form.header" placeholder='{"User-Agent":"..."}' /></label>
      <label>登录界面<input v-model="form.loginUi" /></label>
      <label>JS 库<input v-model="form.jsLib" /></label>
      <label>登录校验 JS<input v-model="form.loginCheckJs" /></label>
      <label class="check"><input v-model="form.enabledCookieJar" type="checkbox" />启用 CookieJar</label>
      <div class="actions">
        <button type="submit">保存</button>
        <button type="button" @click="editing = false">取消</button>
      </div>
    </form>

    <ol class="list">
      <li class="system">
        <div class="text">
          <p class="name">系统 TTS</p>
          <p class="meta">使用宿主系统语音合成，无需配置。</p>
        </div>
        <span class="tag">内置</span>
      </li>
      <li v-for="engine in engines" :key="engine.id">
        <div class="text">
          <p class="name">{{ engine.name }}</p>
          <code>{{ engine.url }}</code>
          <p class="meta">并发率：{{ engine.concurrentRate || "0" }} · {{ engine.contentType }}</p>
        </div>
        <div class="actions">
          <button @click="startEdit(engine)">编辑</button>
          <button class="danger" @click="remove(engine)">删除</button>
        </div>
      </li>
    </ol>
    <p v-if="engines.length === 0" class="empty">暂无在线语音引擎</p>
  </div>
</template>

<style scoped>
.aloud-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  background: var(--ac-background);
}
.toolbar {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  gap: 8px;
}
.toolbar h1 {
  margin: 0 auto 0 0;
  font-size: 18px;
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
}
.hint,
.message {
  font-size: 12px;
  color: #8a8175;
}
.message {
  color: var(--lr-color-primary);
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
.editor input:not([type="checkbox"]) {
  padding: 7px;
  border: 1px solid #ccc;
  border-radius: 8px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.check {
  flex-direction: row !important;
  align-items: center;
  gap: 5px;
}
.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.list {
  list-style: none;
  padding: 0;
  margin: 12px 0 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.list li {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px;
  border: 1px solid #eee7da;
  border-radius: 10px;
  background: var(--lr-color-surface);
}
.list li.system {
  background: #f2efe6;
}
.text {
  flex: 1;
  min-width: 0;
}
.name {
  margin: 0;
  font-weight: 600;
}
code {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  word-break: break-all;
  color: #6b6154;
}
.meta {
  margin: 4px 0 0;
  font-size: 12px;
  color: #8a8175;
}
.tag {
  font-size: 11px;
  color: #6b6154;
  background: #efe8db;
  border-radius: 4px;
  padding: 2px 6px;
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
.empty {
  color: #999;
  text-align: center;
  padding: 24px 0;
}
</style>
