<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import {
  rssSourceDelete,
  rssSourceExport,
  rssSourceImport,
  rssSourceList,
} from "@/core/application/rss";
import type { RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();
const sources = ref<RssSource[]>([]);
const keyword = ref("");
const loading = ref(false);
const message = ref("");

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  if (!key) return sources.value;
  return sources.value.filter((source) => {
    const name = String(source.sourceName || "").toLowerCase();
    const group = String(source.sourceGroup || "").toLowerCase();
    return name.includes(key) || group.includes(key);
  });
});

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2500);
}

async function load() {
  loading.value = true;
  try {
    sources.value = await rssSourceList();
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

function open(source: RssSource) {
  router.push({
    path: "/rss-articles",
    query: { sourceUrl: source.sourceUrl, sourceName: source.sourceName },
  });
}

async function importSource() {
  const path = await platform.dialog.open({
    multiple: false,
    filters: [{ name: "RSS 源 JSON", extensions: ["json", "txt"] }],
  });
  if (typeof path !== "string" || !path) return;
  try {
    const bytes = await platform.files.readFile(path);
    const count = await rssSourceImport(new TextDecoder("utf-8").decode(bytes));
    await load();
    showMessage(`已导入 ${count} 个订阅源`);
  } catch (error) {
    showMessage(String(error));
  }
}

async function exportSource() {
  if (!sources.value.length) {
    showMessage("暂无可导出的订阅源");
    return;
  }
  try {
    const json = await rssSourceExport();
    const path = await platform.dialog.save({
      defaultPath: "rss-sources.json",
      filters: [{ name: "RSS 源 JSON", extensions: ["json"] }],
    });
    if (!path) return;
    await platform.files.writeFile(path, new TextEncoder().encode(json));
    showMessage("订阅源已导出");
  } catch (error) {
    showMessage(String(error));
  }
}

async function remove(source: RssSource) {
  if (!confirm(`删除订阅源「${source.sourceName || source.sourceUrl}」？`)) return;
  try {
    await rssSourceDelete(source.sourceUrl);
    sources.value = sources.value.filter((item) => item.sourceUrl !== source.sourceUrl);
  } catch (error) {
    showMessage(String(error));
  }
}

onMounted(load);
</script>

<template>
  <div class="rss-page">
    <TitleBar title="订阅" :back="false">
      <template #content>
        <label class="search">
          <input v-model="keyword" type="search" placeholder="搜索" />
        </label>
      </template>
      <template #actions>
        <button aria-label="导入" @click="importSource">
          <Icon name="ic_add_online" :size="24" />
        </button>
        <button aria-label="导出" @click="exportSource">
          <Icon name="ic_share" :size="24" />
        </button>
      </template>
    </TitleBar>

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else class="rss-grid">
      <EmptyState
        v-if="!filtered.length"
        icon="ic_bottom_rss_e"
        text="RSS 源目前为空！"
        action="导入订阅源"
        @action="importSource"
      />
      <template v-else>
        <button
          v-for="source in filtered"
          :key="source.sourceUrl"
          class="rss-item"
          @click="open(source)"
          @contextmenu.prevent="remove(source)"
        >
          <span class="rss-name">{{ source.sourceName || source.sourceUrl }}</span>
          <span v-if="source.sourceGroup" class="rss-group">{{ source.sourceGroup }}</span>
        </button>
      </template>
    </main>
  </div>
</template>

<style scoped>
.rss-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.search {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
}
.search input {
  width: 100%;
  height: 30px;
  border: none;
  border-radius: 17px;
  background: rgba(255, 255, 255, 0.22);
  color: #fff;
  font-size: 13px;
  padding: 0 16px;
  outline: none;
}
.search input::placeholder {
  color: rgba(255, 255, 255, 0.65);
}
.message,
.loading {
  margin: 8px 16px;
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.rss-grid {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  align-content: start;
}
.rss-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 18px 8px;
  border: none;
  background: transparent;
  cursor: pointer;
}
.rss-name {
  font-size: 13px;
  color: var(--ac-primary-text);
  text-align: center;
  word-break: break-all;
}
.rss-group {
  font-size: 11px;
  color: var(--ac-secondary-text);
}
</style>
