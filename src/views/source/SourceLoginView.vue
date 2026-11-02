<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  bookSourceClearCookies,
  bookSourceGet,
  bookSourceSaveCookies,
  webviewLogin,
} from "@/core/application/bookSources";
import { rssSourceList } from "@/core/application/rss";
import { platform } from "@/runtime";
import type { BookSource, RssSource, WebViewLoginResult } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.url || ""));
const sourceType = computed(() => String(route.query.type || "bookSource"));
const source = ref<BookSource | RssSource | null>(null);
const loginUrl = ref("");
const waitSeconds = ref(60);
const running = ref(false);
const message = ref("");
const result = ref<WebViewLoginResult | null>(null);
const sourceLabel = computed(() => {
  if (!source.value) return sourceUrl.value;
  if ("bookSourceName" in source.value) return source.value.bookSourceName;
  return source.value.sourceName;
});

async function load() {
  if (!sourceUrl.value) return;
  try {
    if (sourceType.value === "rss") {
      source.value =
        (await rssSourceList()).find((item) => item.sourceUrl === sourceUrl.value) ?? null;
    } else {
      source.value = await bookSourceGet(sourceUrl.value);
    }
    loginUrl.value = String(source.value?.loginUrl || sourceUrl.value);
  } catch (error) {
    message.value = String(error);
  }
}

async function login() {
  const target = loginUrl.value.trim();
  if (!target || running.value) return;
  if (!platform.capabilities.webview) {
    message.value = "当前运行时不支持 WebView 登录";
    return;
  }
  running.value = true;
  message.value = "";
  result.value = null;
  try {
    const response = await webviewLogin(target, Math.max(5, waitSeconds.value) * 1000);
    result.value = response;
    if (response.cookies.length) {
      await bookSourceSaveCookies(sourceUrl.value, response.cookies, response.url || target);
      message.value = `登录完成，已保存 ${response.cookies.length} 个 Cookie`;
    } else {
      message.value = "登录完成，但没有读取到 Cookie";
    }
  } catch (error) {
    message.value = String(error);
  } finally {
    running.value = false;
  }
}

async function clearCookies() {
  if (!sourceUrl.value) return;
  try {
    await bookSourceClearCookies(sourceUrl.value);
    result.value = null;
    message.value = "Cookie 已清除";
  } catch (error) {
    message.value = String(error);
  }
}

onMounted(load);
</script>

<template>
  <div class="login-page">
    <TitleBar :title="sourceType === 'rss' ? '订阅源登录' : '书源登录'" @back="router.back()" />
    <main class="content">
      <p class="source-name">{{ sourceLabel }}</p>
      <label>
        登录地址
        <input v-model="loginUrl" placeholder="https://example.com/login" />
      </label>
      <label>
        登录窗口停留秒数
        <input v-model.number="waitSeconds" type="number" min="5" max="300" />
      </label>
      <p class="hint">登录窗口打开后请完成登录；窗口会在设定时间后自动关闭并保存 Cookie。</p>
      <div class="row">
        <button :disabled="running || !loginUrl" @click="login">
          {{ running ? "等待登录窗口…" : "打开登录窗口" }}
        </button>
        <button :disabled="!sourceUrl" @click="clearCookies">清除 Cookie</button>
      </div>
      <p v-if="message" class="message">{{ message }}</p>
      <section v-if="result" class="result">
        <h3>登录结果</h3>
        <p class="url">{{ result.url }}</p>
        <p v-if="!result.cookies.length" class="empty">没有 Cookie</p>
        <div v-for="(cookie, index) in result.cookies" :key="index" class="cookie">
          {{ cookie }}
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.login-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.source-name {
  margin: 0;
  font-size: 15px;
  color: var(--ac-primary-text);
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
input {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px 10px;
  border-radius: 4px;
}
.hint,
.message {
  margin: 0;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.row {
  display: flex;
  gap: 8px;
}
.row button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 8px 12px;
}
.result {
  border-top: 1px solid var(--ac-divider);
  padding-top: 10px;
}
.result h3 {
  margin: 0 0 6px;
  font-size: 13px;
}
.url,
.cookie {
  font-size: 12px;
  color: var(--ac-secondary-text);
  word-break: break-all;
}
.cookie {
  padding: 3px 0;
}
.empty {
  margin: 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
</style>
