<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import {
  bookSourceDelete,
  bookSourceExport,
  bookSourceImport,
  bookSourceList,
  bookSourceSetEnabled,
} from "@/core/application/bookSources";
import type { BookSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();
const sources = ref<BookSource[]>([]);
const keyword = ref("");
const message = ref("");
const loading = ref(false);

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  if (!key) return sources.value;
  return sources.value.filter((source) => {
    const name = String(source.bookSourceName || "").toLowerCase();
    const group = String(source.bookSourceGroup || "").toLowerCase();
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
    sources.value = await bookSourceList();
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

function edit(source?: BookSource) {
  router.push({
    path: "/book-source-edit",
    query: source ? { url: source.bookSourceUrl } : {},
  });
}

async function toggle(source: BookSource) {
  const next = !source.enabled;
  try {
    await bookSourceSetEnabled(source.bookSourceUrl, next);
    source.enabled = next;
  } catch (error) {
    showMessage(String(error));
  }
}

async function remove(source: BookSource) {
  if (!confirm(`删除书源「${source.bookSourceName || source.bookSourceUrl}」？`)) {
    return;
  }
  try {
    await bookSourceDelete(source.bookSourceUrl);
    sources.value = sources.value.filter(
      (item) => item.bookSourceUrl !== source.bookSourceUrl,
    );
  } catch (error) {
    showMessage(String(error));
  }
}

async function importSource() {
  const path = await platform.dialog.open({
    multiple: false,
    filters: [{ name: "书源 JSON", extensions: ["json", "txt"] }],
  });
  if (typeof path !== "string" || !path) return;
  try {
    const bytes = await platform.files.readFile(path);
    const json = new TextDecoder("utf-8").decode(bytes);
    const count = await bookSourceImport(json);
    await load();
    showMessage(`已导入 ${count} 个书源`);
  } catch (error) {
    showMessage(String(error));
  }
}

async function exportSource() {
  if (!sources.value.length) {
    showMessage("暂无可导出的书源");
    return;
  }
  try {
    const json = await bookSourceExport();
    const path = await platform.dialog.save({
      defaultPath: "book-sources.json",
      filters: [{ name: "书源 JSON", extensions: ["json"] }],
    });
    if (!path) return;
    await platform.files.writeFile(path, new TextEncoder().encode(json));
    showMessage("书源已导出");
  } catch (error) {
    showMessage(String(error));
  }
}

onMounted(load);
</script>

<template>
  <div class="source-page">
    <TitleBar title="书源" @back="router.back()">
      <template #content>
        <input v-model="keyword" class="search" placeholder="搜索书源" />
      </template>
      <template #actions>
        <button aria-label="新增" @click="edit()">
          <Icon name="ic_add" :size="24" />
        </button>
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
    <main v-else class="list">
      <EmptyState
        v-if="!filtered.length"
        icon="ic_cfg_source"
        text="暂无书源"
        action="新增书源"
        @action="edit()"
      />
      <template v-else>
        <div v-for="source in filtered" :key="source.bookSourceUrl" class="row">
          <label class="check">
            <input
              type="checkbox"
              :checked="source.enabled"
              @change="toggle(source)"
            />
          </label>
          <button class="info" @click="edit(source)">
            <span class="name">{{ source.bookSourceName || source.bookSourceUrl }}</span>
            <span class="meta">
              {{ source.bookSourceGroup || "未分组" }}
              · {{ source.bookSourceType === 0 ? "文本" : source.bookSourceType === 1 ? "音频" : source.bookSourceType === 2 ? "图片" : "文件" }}
            </span>
          </button>
          <button class="mini" @click="edit(source)">编辑</button>
          <button class="mini danger" @click="remove(source)">删除</button>
        </div>
      </template>
    </main>
  </div>
</template>

<style scoped>
.source-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.search {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
  padding: 6px 10px;
  border: none;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
  outline: none;
}
.search::placeholder {
  color: rgba(255, 255, 255, 0.6);
}
.message,
.loading {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  overflow-y: auto;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.check {
  display: flex;
  align-items: center;
}
.info {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
  padding: 0;
}
.name,
.meta {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.meta {
  margin-top: 2px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.mini {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
}
.mini.danger {
  border-color: var(--ac-error);
  color: var(--ac-error);
}
</style>
