<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  rssArticleList,
  rssContent,
  rssReadRecordAdd,
  rssSourceList,
  rssStarGet,
  rssStarSave,
} from "@/core/application/rss";
import { platform } from "@/runtime";
import type { RssArticle, RssSource, RssStar } from "@/core/domain";
import { sanitizeHtml } from "@/utils/ruleQuery";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import RssFavoriteDialog from "./RssFavoriteDialog.vue";

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.sourceUrl || ""));
const link = computed(() => String(route.query.link || ""));
const origin = computed(() => String(route.query.origin || sourceUrl.value));
const title = computed(() => String(route.query.title || "订阅文章"));
const sortName = computed(() => String(route.query.sort || ""));
const source = ref<RssSource | null>(null);
const article = ref<RssArticle | null>(null);
const html = ref("");
const loading = ref(false);
const message = ref("");
const menuVisible = ref(false);
const favoriteVisible = ref(false);
const star = ref<RssStar | null>(null);
const speaking = ref(false);

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "refresh", label: "刷新" },
  { key: "star", label: "收藏" },
  { key: "share", label: "分享" },
  { key: "aloud", label: speaking.value ? "停止朗读" : "朗读" },
  ...(platform.capabilities.webview && String(source.value?.loginUrl || "").trim()
    ? [{ key: "login", label: "登录" }]
    : []),
  { key: "browser", label: "浏览器打开" },
]);

function absoluteUrl(base: string, url: string): string {
  try {
    return new URL(url, base).toString();
  } catch {
    return url;
  }
}

function prepareHtml(input: string, base: string): string {
  const safe = sanitizeHtml(input);
  const doc = new DOMParser().parseFromString(safe, "text/html");
  doc.querySelectorAll("script,style,iframe,object,embed,form").forEach((node) => node.remove());
  doc.querySelectorAll("*").forEach((node) => {
    for (const attribute of [...node.attributes]) {
      if (/^on/i.test(attribute.name)) node.removeAttribute(attribute.name);
    }
  });
  doc.querySelectorAll("img[src]").forEach((node) => {
    const src = node.getAttribute("src") || "";
    node.setAttribute("src", absoluteUrl(base, src));
  });
  doc.querySelectorAll("a[href]").forEach((node) => {
    const href = node.getAttribute("href") || "";
    node.setAttribute("href", absoluteUrl(base, href));
    node.setAttribute("target", "_blank");
    node.setAttribute("rel", "noreferrer");
  });
  return doc.body.innerHTML;
}

async function load() {
  loading.value = true;
  message.value = "";
  try {
    const sources = await rssSourceList();
    source.value = sources.find((item) => item.sourceUrl === sourceUrl.value) ?? null;
    if (!source.value) throw new Error("订阅源不存在");
    const cached = await rssArticleList(sourceUrl.value, sortName.value);
    article.value =
      cached.find((item) => item.link === link.value) ??
      ({
        origin: origin.value,
        sort: sortName.value,
        title: title.value,
        link: link.value,
        group: "默认分组",
        read: true,
      } as RssArticle);
    const content = await rssContent(
      sourceUrl.value,
      article.value,
      source.value.ruleContent ?? null,
    );
    html.value = prepareHtml(
      content || article.value.description || "",
      article.value.link || sourceUrl.value,
    );
    star.value = await rssStarGet(article.value.origin, article.value.link);
    await rssReadRecordAdd(article.value.link, article.value.title);
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function toggleStar() {
  if (!article.value) return;
  if (!star.value) {
    await rssStarSave(article.value);
    star.value = await rssStarGet(article.value.origin, article.value.link);
  }
  favoriteVisible.value = true;
}

async function reloadStar() {
  if (!article.value) return;
  star.value = await rssStarGet(article.value.origin, article.value.link);
}

function toggleAloud() {
  if (!window.speechSynthesis) {
    message.value = "当前环境不支持语音合成";
    return;
  }
  if (speaking.value) {
    window.speechSynthesis.cancel();
    speaking.value = false;
    return;
  }
  const text = new DOMParser()
    .parseFromString(html.value, "text/html")
    .body.textContent?.trim();
  if (!text) return;
  const utterance = new SpeechSynthesisUtterance(text);
  utterance.lang = "zh-CN";
  utterance.onend = () => {
    speaking.value = false;
  };
  utterance.onerror = () => {
    speaking.value = false;
  };
  speaking.value = true;
  window.speechSynthesis.speak(utterance);
}

async function onMenu(key: string) {
  if (key === "refresh") await load();
  else if (key === "star") await toggleStar();
  else if (key === "share") {
    const url = article.value?.link || link.value;
    if (navigator.share) await navigator.share({ title: title.value, url }).catch(() => undefined);
    else {
      await navigator.clipboard.writeText(url);
      message.value = "链接已复制";
    }
  } else if (key === "aloud") toggleAloud();
  else if (key === "login") {
    if (!platform.capabilities.webview) {
      message.value = "当前运行时不支持 WebView 登录";
      return;
    }
    router.push({ path: "/source-login", query: { type: "rss", url: sourceUrl.value } });
  } else if (key === "browser") {
    window.open(article.value?.link || link.value, "_blank", "noopener,noreferrer");
  }
}

onMounted(load);

onBeforeUnmount(() => {
  window.speechSynthesis?.cancel();
});
</script>

<template>
  <div class="rss-read-page">
    <TitleBar :title="article?.title || title" @back="router.back()">
      <template #actions>
        <button aria-label="收藏" @click="toggleStar">
          <Icon :name="star ? 'ic_star' : 'ic_star_border'" :size="24" />
        </button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />
    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="status">加载中…</p>
    <main v-else class="content" v-html="html"></main>

    <RssFavoriteDialog
      v-model="favoriteVisible"
      :article="article"
      @changed="reloadStar"
    />
  </div>
</template>

<style scoped>
.rss-read-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.message,
.status {
  margin: 8px 16px;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 12px 18px 48px;
  color: var(--ac-primary-text);
  font-size: 17px;
  line-height: 1.8;
  word-break: break-word;
}
.content :deep(img) {
  max-width: 100%;
  height: auto;
}
.content :deep(a) {
  color: var(--ac-primary);
}
.content :deep(pre) {
  white-space: pre-wrap;
}
</style>
