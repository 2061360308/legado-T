<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { onBeforeRouteLeave, useRoute, useRouter } from "vue-router";
import {
  rssSourceList,
  rssSourceSave,
} from "@/core/application/rss";
import { bookSourceClearCookies } from "@/core/application/bookSources";
import { platform } from "@/runtime";
import type { RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import QrCodeDialog from "@/views/source/QrCodeDialog.vue";

type TabKey = "basic" | "list" | "webview";

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.sourceUrl || ""));
const draft = ref<RssSource>(emptySource());
const original = ref("");
const originalUrl = ref("");
const activeTab = ref<TabKey>("basic");
const menuVisible = ref(false);
const qrImportVisible = ref(false);
const qrShareVisible = ref(false);
const autoComplete = ref(false);
const message = ref("");
const loading = ref(false);
const saving = ref(false);

const tabs: { key: TabKey; label: string }[] = [
  { key: "basic", label: "基础" },
  { key: "list", label: "列表" },
  { key: "webview", label: "WEB_VIEW" },
];

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "save", label: "保存" },
  { key: "debug", label: "调试" },
  ...(platform.capabilities.webview && String(draft.value.loginUrl || "").trim()
    ? [{ key: "login", label: "登录" }]
    : []),
  { key: "variables", label: "设置源变量" },
  { key: "clearCookie", label: "清除 Cookie" },
  { key: "autoComplete", label: "自动补全", checked: autoComplete.value },
  { key: "copy", label: "复制订阅源" },
  { key: "paste", label: "粘贴订阅源" },
  { key: "qrImport", label: "二维码导入" },
  { key: "shareText", label: "分享文本" },
  { key: "shareQr", label: "分享二维码" },
  { key: "help", label: "帮助" },
]);

const dirty = computed(
  () => Boolean(original.value) && JSON.stringify(draft.value) !== original.value,
);

function emptySource(): RssSource {
  return {
    sourceUrl: "",
    sourceName: "",
    sourceIcon: "",
    sourceGroup: "",
    sourceComment: "",
    enabled: true,
    enabledCookieJar: true,
    singleUrl: false,
    articleStyle: 0,
    customOrder: Date.now(),
    lastUpdateTime: 0,
    enableJs: true,
    loadWithBaseUrl: true,
  };
}

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2600);
}

async function load() {
  loading.value = true;
  try {
    if (!sourceUrl.value) {
      draft.value = emptySource();
      originalUrl.value = "";
      original.value = JSON.stringify(draft.value);
      return;
    }
    const source = (await rssSourceList()).find(
      (item) => item.sourceUrl === sourceUrl.value,
    );
    draft.value = source ? { ...source } : { ...emptySource(), sourceUrl: sourceUrl.value };
    originalUrl.value = source?.sourceUrl || "";
    original.value = JSON.stringify(draft.value);
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

async function persist(): Promise<boolean> {
  if (saving.value) return false;
  const url = String(draft.value.sourceUrl || "").trim();
  const name = String(draft.value.sourceName || "").trim();
  if (!url || !name) {
    showMessage("名称和 URL 不能为空");
    return false;
  }
  saving.value = true;
  try {
    const next: RssSource = {
      ...draft.value,
      sourceUrl: url,
      sourceName: name,
      sourceGroup: String(draft.value.sourceGroup || "").trim() || null,
      lastUpdateTime: Math.floor(Date.now() / 1000),
    };
    await rssSourceSave(
      next,
      originalUrl.value && originalUrl.value !== next.sourceUrl ? originalUrl.value : null,
    );
    draft.value = next;
    originalUrl.value = next.sourceUrl;
    original.value = JSON.stringify(next);
    return true;
  } catch (error) {
    showMessage(String(error));
    return false;
  } finally {
    saving.value = false;
  }
}

async function save() {
  if (await persist()) router.back();
}

async function saveAndGo(path: string, query: Record<string, string> = {}) {
  if (!(await persist())) return;
  router.push({ path, query: { ...query, url: draft.value.sourceUrl } });
}

async function onMenu(key: string) {
  if (key === "save") await save();
  else if (key === "debug") {
    await saveAndGo("/rss-source-debug", { sourceUrl: draft.value.sourceUrl });
  } else if (key === "login") {
    if (!platform.capabilities.webview) {
      showMessage("当前运行时不支持 WebView 登录");
      return;
    }
    await saveAndGo("/source-login", { type: "rss" });
  } else if (key === "variables") {
    await saveAndGo("/book-source-variables", { type: "rss" });
  } else if (key === "clearCookie") {
    await bookSourceClearCookies(draft.value.sourceUrl);
    showMessage("Cookie 已清除");
  } else if (key === "autoComplete") {
    autoComplete.value = !autoComplete.value;
  } else if (key === "copy") {
    await navigator.clipboard.writeText(JSON.stringify(draft.value, null, 2));
    showMessage("订阅源已复制");
  } else if (key === "paste") {
    try {
      const parsed = JSON.parse(await navigator.clipboard.readText()) as RssSource;
      draft.value = { ...emptySource(), ...parsed };
      showMessage("已粘贴订阅源");
    } catch (error) {
      showMessage(`粘贴失败：${error}`);
    }
  } else if (key === "qrImport") {
    qrImportVisible.value = true;
  } else if (key === "shareText") {
    const text = JSON.stringify(draft.value, null, 2);
    if (navigator.share) await navigator.share({ text }).catch(() => undefined);
    else {
      await navigator.clipboard.writeText(text);
      showMessage("订阅源 JSON 已复制");
    }
  } else if (key === "shareQr") {
    qrShareVisible.value = true;
  } else if (key === "help") {
    showMessage("订阅源规则帮助暂未接入");
  }
}

function importQr(text: string) {
  try {
    const parsed = JSON.parse(text) as RssSource;
    draft.value = { ...emptySource(), ...parsed };
    showMessage("二维码订阅源已载入");
  } catch (error) {
    showMessage(`二维码内容无效：${error}`);
  }
}

onMounted(load);

onBeforeRouteLeave(() => {
  if (!dirty.value) return true;
  return confirm("订阅源尚未保存，确定退出？");
});
</script>

<template>
  <div class="source-edit">
    <TitleBar :title="sourceUrl ? '编辑订阅源' : '新建订阅源'" @back="router.back()">
      <template #actions>
        <button aria-label="保存" :disabled="saving || loading" @click="save">
          <Icon name="ic_save" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />

    <div class="top-row">
      <label><input v-model="draft.enabled" type="checkbox" /> 启用</label>
      <label><input v-model="draft.singleUrl" type="checkbox" /> 单 URL</label>
      <label><input v-model="draft.enabledCookieJar" type="checkbox" /> 自动保存 Cookie</label>
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

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else class="form">
      <template v-if="activeTab === 'basic'">
        <label>名称<input v-model="draft.sourceName" /></label>
        <label>URL<input v-model="draft.sourceUrl" placeholder="https://example.com/rss" /></label>
        <label>图标<input v-model="draft.sourceIcon" placeholder="https://example.com/icon.png" /></label>
        <label>分组<input v-model="draft.sourceGroup" placeholder="分组1,分组2" /></label>
        <label>注释<textarea v-model="draft.sourceComment" /></label>
        <label>分类 URL<input v-model="draft.sortUrl" placeholder="https://example.com/sort" /></label>
        <label>登录 URL<input v-model="draft.loginUrl" /></label>
        <label>登录 UI<input v-model="draft.loginUi" /></label>
        <label>封面解密 JS<textarea v-model="draft.coverDecodeJs" /></label>
        <label>请求头<textarea v-model="draft.header" placeholder='{"User-Agent":"..."}' /></label>
        <label>变量说明<textarea v-model="draft.variableComment" /></label>
        <label>并发率<input v-model="draft.concurrentRate" placeholder="1000" /></label>
        <label>jsLib<textarea v-model="draft.jsLib" class="code" /></label>
      </template>

      <template v-else-if="activeTab === 'list'">
        <label>文章列表规则<textarea v-model="draft.ruleArticles" class="code" /></label>
        <label>下一页规则<textarea v-model="draft.ruleNextPage" class="code" /></label>
        <label>标题规则<textarea v-model="draft.ruleTitle" class="code" /></label>
        <label>发布日期规则<textarea v-model="draft.rulePubDate" class="code" /></label>
        <label>描述规则<textarea v-model="draft.ruleDescription" class="code" /></label>
        <label>图片规则<textarea v-model="draft.ruleImage" class="code" /></label>
        <label>链接规则<textarea v-model="draft.ruleLink" class="code" /></label>
      </template>

      <template v-else>
        <label class="check"><input v-model="draft.enableJs" type="checkbox" /> 启用 JS</label>
        <label class="check"><input v-model="draft.loadWithBaseUrl" type="checkbox" /> 使用 BaseURL</label>
        <label>正文规则<textarea v-model="draft.ruleContent" class="code" /></label>
        <label>样式<textarea v-model="draft.style" class="code" /></label>
        <label>注入 JS<textarea v-model="draft.injectJs" class="code" /></label>
        <label>内容白名单<textarea v-model="draft.contentWhitelist" class="code" /></label>
        <label>内容黑名单<textarea v-model="draft.contentBlacklist" class="code" /></label>
        <label>URL 跳转拦截<textarea v-model="draft.shouldOverrideUrlLoading" class="code" /></label>
      </template>
    </main>

    <QrCodeDialog v-model="qrImportVisible" mode="import" @import="importQr" />
    <QrCodeDialog
      v-model="qrShareVisible"
      mode="share"
      :text="JSON.stringify(draft)"
    />
  </div>
</template>

<style scoped>
.source-edit {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.top-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--ac-divider);
  color: var(--ac-primary-text);
  font-size: 13px;
}
.top-row label {
  display: flex;
  align-items: center;
  gap: 6px;
}
.tabs {
  display: flex;
  border-bottom: 1px solid var(--ac-divider);
  overflow-x: auto;
}
.tabs button {
  flex: 1;
  min-width: 90px;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 11px 8px;
  font-size: 13px;
  border-bottom: 2px solid transparent;
}
.tabs button.active {
  color: var(--ac-primary);
  border-bottom-color: var(--ac-primary);
}
.message,
.loading {
  margin: 8px 16px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.form {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 12px 16px 40px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.form label.check {
  flex-direction: row;
  align-items: center;
  gap: 8px;
}
.form input,
.form textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 9px 10px;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: var(--ac-background);
  color: var(--ac-primary-text);
  font-size: 13px;
}
.form textarea {
  min-height: 64px;
  resize: vertical;
  font-family: inherit;
}
.form textarea.code {
  min-height: 96px;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 12px;
}
</style>
