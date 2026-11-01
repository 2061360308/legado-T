<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  bookSourceGet,
  bookSourceSave,
} from "@/core/application/bookSources";
import type { BookSource, BookSourceRule } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import UrlOptionDialog from "./UrlOptionDialog.vue";
import QrCodeDialog from "./QrCodeDialog.vue";

type RuleKey = "search" | "explore" | "bookInfo" | "toc" | "content" | "review";

const route = useRoute();
const router = useRouter();
const url = String(route.query.url || "");
const activeTab = ref<"basic" | RuleKey | "json">("basic");
const draft = ref<BookSource>(emptySource());
const rawJson = ref("");
const message = ref("");
const loading = ref(false);
const saving = ref(false);
const urlOptionVisible = ref(false);
const urlOptionJson = ref("");
const qrVisible = ref(false);
const qrText = computed(() => JSON.stringify(draft.value, null, 2));

const tabs: { key: "basic" | RuleKey | "json"; label: string }[] = [
  { key: "basic", label: "基础" },
  { key: "search", label: "搜索" },
  { key: "explore", label: "发现" },
  { key: "bookInfo", label: "详情" },
  { key: "toc", label: "目录" },
  { key: "content", label: "正文" },
  { key: "review", label: "评论" },
  { key: "json", label: "JSON" },
];

const ruleTexts = reactive<Record<RuleKey, string>>({
  search: "",
  explore: "",
  bookInfo: "",
  toc: "",
  content: "",
  review: "",
});

const ruleTargets: Record<RuleKey, keyof BookSource> = {
  search: "ruleSearch",
  explore: "ruleExplore",
  bookInfo: "ruleBookInfo",
  toc: "ruleToc",
  content: "ruleContent",
  review: "ruleReview",
};

const activeRule = computed(() =>
  activeTab.value === "basic" || activeTab.value === "json"
    ? null
    : activeTab.value,
);

function emptySource(): BookSource {
  return {
    bookSourceUrl: "",
    bookSourceName: "",
    bookSourceGroup: "",
    bookSourceType: 0,
    enabled: true,
    enabledExplore: true,
    enabledCookieJar: false,
    customOrder: Date.now(),
    weight: 0,
    lastUpdateTime: 0,
    respondTime: 180000,
  };
}

function ruleText(value: unknown): string {
  if (!value || typeof value !== "object") return "";
  if (Object.keys(value as Record<string, unknown>).length === 0) return "";
  return JSON.stringify(value, null, 2);
}

function fill(source: BookSource) {
  draft.value = JSON.parse(JSON.stringify(source));
  for (const key of Object.keys(ruleTexts) as RuleKey[]) {
    ruleTexts[key] = ruleText(draft.value[ruleTargets[key]]);
  }
  rawJson.value = JSON.stringify(draft.value, null, 2);
}

async function load() {
  if (!url) {
    fill(emptySource());
    return;
  }
  loading.value = true;
  try {
    const source = await bookSourceGet(url);
    fill(source ?? emptySource());
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

function parseRule(key: RuleKey): BookSourceRule | null {
  const text = ruleTexts[key].trim();
  if (!text) return null;
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    throw new Error(`${tabs.find((tab) => tab.key === key)?.label || key}规则 JSON 无效：${error}`);
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`${tabs.find((tab) => tab.key === key)?.label || key}规则必须是 JSON 对象`);
  }
  return Object.keys(parsed as Record<string, unknown>).length
    ? (parsed as BookSourceRule)
    : null;
}

async function persist(): Promise<boolean> {
  if (saving.value) return false;
  message.value = "";
  try {
    const next: BookSource = JSON.parse(JSON.stringify(draft.value));
    next.bookSourceName = String(next.bookSourceName || "").trim();
    next.bookSourceUrl = String(next.bookSourceUrl || "").trim();
    next.bookSourceGroup = String(next.bookSourceGroup || "").trim() || null;
    next.bookSourceType = Number(next.bookSourceType) || 0;
    if (!next.bookSourceUrl) throw new Error("书源 URL 不能为空");
    if (!next.bookSourceName) throw new Error("书源名称不能为空");
    for (const key of Object.keys(ruleTexts) as RuleKey[]) {
      (next as Record<string, unknown>)[ruleTargets[key]] = parseRule(key);
    }
    saving.value = true;
    await bookSourceSave(next);
    draft.value = next;
    rawJson.value = JSON.stringify(next, null, 2);
    return true;
  } catch (error) {
    message.value = String(error);
    return false;
  } finally {
    saving.value = false;
  }
}

async function save() {
  if (await persist()) router.back();
}

async function saveAndGo(path: string) {
  if (!draft.value.bookSourceUrl) {
    message.value = "请先填写书源 URL";
    return;
  }
  if (await persist()) {
    router.push({ path, query: { url: draft.value.bookSourceUrl } });
  }
}

function openDebug() {
  void saveAndGo("/book-source-debug");
}

function openLogin() {
  void saveAndGo("/source-login");
}

function openVariables() {
  void saveAndGo("/book-source-variables");
}

function applyUrlOption(json: string) {
  urlOptionJson.value = json;
}

function appendUrlOption(field: "searchUrl" | "exploreUrl") {
  if (!urlOptionJson.value.trim()) return;
  const current = String(draft.value[field] || "").trim();
  if (!current) {
    message.value = "请先填写目标 URL";
    return;
  }
  draft.value[field] = `${current},${urlOptionJson.value}`;
  message.value = "URL 参数已追加";
}

async function copyUrlOption() {
  if (!urlOptionJson.value.trim()) return;
  await navigator.clipboard.writeText(urlOptionJson.value);
  message.value = "URL 参数已复制";
}

async function pasteSource() {
  try {
    const text = await navigator.clipboard.readText();
    if (!text.trim()) return;
    const parsed = JSON.parse(text) as BookSource;
    fill(parsed);
    message.value = "已粘贴书源";
  } catch (error) {
    message.value = `粘贴失败：${error}`;
  }
}

async function copySource() {
  try {
    await navigator.clipboard.writeText(JSON.stringify(draft.value, null, 2));
    message.value = "书源已复制";
  } catch (error) {
    message.value = String(error);
  }
}

function applyRawJson() {
  try {
    const parsed = JSON.parse(rawJson.value) as BookSource;
    fill(parsed);
    message.value = "已从 JSON 应用";
  } catch (error) {
    message.value = `JSON 无效：${error}`;
  }
}

function refreshRawJson() {
  rawJson.value = JSON.stringify(draft.value, null, 2);
}

onMounted(load);
</script>

<template>
  <div class="source-edit">
    <TitleBar :title="url ? '编辑书源' : '新增书源'" @back="router.back()">
      <template #actions>
        <button aria-label="保存" :disabled="saving || loading" @click="save">
          <Icon name="ic_save" :size="24" />
        </button>
      </template>
    </TitleBar>

    <div class="top-row">
      <span class="label">书源类型</span>
      <select v-model.number="draft.bookSourceType">
        <option :value="0">文本</option>
        <option :value="1">音频</option>
        <option :value="2">图片</option>
        <option :value="3">文件</option>
      </select>
      <label><input v-model="draft.enabled" type="checkbox" /> 启用</label>
      <label><input v-model="draft.enabledExplore" type="checkbox" /> 发现</label>
      <label><input v-model="draft.enabledCookieJar" type="checkbox" /> 自动保存 Cookie</label>
      <span class="spacer" />
      <button class="mini" @click="openDebug">调试</button>
      <button class="mini" @click="openLogin">登录</button>
      <button class="mini" @click="openVariables">变量</button>
      <button class="mini" @click="qrVisible = true">二维码</button>
    </div>

    <nav class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        :class="{ active: activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </nav>

    <main class="form">
      <template v-if="activeTab === 'basic'">
        <div class="grid2">
          <label>书源名称<input v-model="draft.bookSourceName" placeholder="书源名称" /></label>
          <label>书源分组<input v-model="draft.bookSourceGroup" placeholder="未分组" /></label>
          <label class="wide">
            书源 URL
            <input v-model="draft.bookSourceUrl" placeholder="https://example.com" />
          </label>
          <label>搜索 URL<input v-model="draft.searchUrl" placeholder="可选" /></label>
          <label>发现 URL<input v-model="draft.exploreUrl" placeholder="可选" /></label>
          <label>发现筛选<input v-model="draft.exploreScreen" placeholder="可选" /></label>
          <label>书籍 URL 正则<input v-model="draft.bookUrlPattern" placeholder="可选" /></label>
          <label>登录 URL<input v-model="draft.loginUrl" placeholder="可选" /></label>
          <label>并发率<input v-model="draft.concurrentRate" placeholder="可选" /></label>
          <label>权重<input v-model.number="draft.weight" type="number" /></label>
          <label>排序<input v-model.number="draft.customOrder" type="number" /></label>
          <label class="wide">请求头 JSON<textarea v-model="draft.header" class="small" placeholder='{"User-Agent":"..."}' /></label>
          <label class="wide">JS Lib<textarea v-model="draft.jsLib" class="small" placeholder="可选" /></label>
          <label class="wide">登录检查 JS<textarea v-model="draft.loginCheckJs" class="small" placeholder="可选" /></label>
          <label class="wide">封面解密 JS<textarea v-model="draft.coverDecodeJs" class="small" placeholder="可选" /></label>
          <label class="wide">注释<textarea v-model="draft.bookSourceComment" class="small" placeholder="可选" /></label>
        </div>
        <div class="row">
          <button @click="pasteSource">粘贴书源</button>
          <button @click="copySource">复制书源</button>
          <button @click="urlOptionVisible = true">URL 选项</button>
        </div>
        <label v-if="urlOptionJson" class="code">
          URL 参数 JSON
          <textarea v-model="urlOptionJson" class="small" />
        </label>
        <div v-if="urlOptionJson" class="row">
          <button @click="appendUrlOption('searchUrl')">追加到搜索 URL</button>
          <button @click="appendUrlOption('exploreUrl')">追加到发现 URL</button>
          <button @click="copyUrlOption">复制参数</button>
        </div>
      </template>

      <template v-else-if="activeTab === 'json'">
        <label class="code">
          完整 JSON
          <textarea v-model="rawJson" spellcheck="false" />
        </label>
        <div class="row">
          <button @click="applyRawJson">应用到表单</button>
          <button @click="refreshRawJson">从表单刷新</button>
          <button @click="copySource">复制</button>
        </div>
      </template>

      <template v-else-if="activeRule">
        <p class="hint">{{ tabs.find((tab) => tab.key === activeRule)?.label }}规则，JSON 对象；留空表示不使用。</p>
        <label class="code">
          {{ activeRule }}
          <textarea v-model="ruleTexts[activeRule]" spellcheck="false" placeholder="{ ... }" />
        </label>
      </template>

      <p v-if="message" class="message">{{ message }}</p>
      <p v-if="loading" class="loading">加载中…</p>
    </main>

    <UrlOptionDialog
      v-model="urlOptionVisible"
      @confirm="applyUrlOption"
    />
    <QrCodeDialog v-model="qrVisible" mode="share" :text="qrText" />
  </div>
</template>

<style scoped>
.source-edit {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.top-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
  font-size: 13px;
  white-space: nowrap;
}
.label,
.hint {
  color: var(--ac-secondary-text);
}
.spacer {
  flex: 1;
}
select {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 4px 6px;
}
.top-row label {
  display: flex;
  align-items: center;
  gap: 4px;
}
input[type="checkbox"] {
  accent-color: var(--ac-primary);
}
.mini,
.row button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 4px 10px;
}
.tabs {
  display: flex;
  gap: 2px;
  padding: 0 6px;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
}
.tabs button {
  border: none;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 8px 10px;
  white-space: nowrap;
  font-size: 13px;
}
.tabs button.active {
  color: var(--ac-primary);
  border-bottom-color: var(--ac-primary);
}
.form {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px 24px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.grid2 {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.grid2 .wide {
  grid-column: 1 / -1;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
input:not([type="checkbox"]),
textarea {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px 10px;
  border-radius: 2px;
  font-size: 14px;
}
textarea.small {
  min-height: 64px;
  resize: vertical;
  font-family: Consolas, monospace;
  font-size: 12px;
}
.code {
  flex: 1;
}
.code textarea {
  min-height: 420px;
  resize: vertical;
  font-family: Consolas, monospace;
  font-size: 13px;
}
.row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.message {
  margin: 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.loading {
  margin: 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
</style>
