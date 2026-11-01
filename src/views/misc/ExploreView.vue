<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { bookSourceList } from "@/core/application/bookSources";
import type { BookSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();
const sources = ref<BookSource[]>([]);
const keyword = ref("");
const loading = ref(false);
const message = ref("");

const exploreSources = computed(() =>
  sources.value.filter(
    (source) =>
      source.enabledExplore !== false &&
      Boolean(String(source.exploreUrl || "").trim()),
  ),
);

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  if (!key) return exploreSources.value;
  return exploreSources.value.filter((source) => {
    const name = String(source.bookSourceName || "").toLowerCase();
    const group = String(source.bookSourceGroup || "").toLowerCase();
    return name.includes(key) || group.includes(key);
  });
});

async function load() {
  loading.value = true;
  try {
    sources.value = await bookSourceList();
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

function open(source: BookSource) {
  router.push({
    path: "/explore-show",
    query: {
      sourceUrl: source.bookSourceUrl,
      sourceName: source.bookSourceName,
    },
  });
}

onMounted(load);
</script>

<template>
  <div class="explore-page">
    <TitleBar title="发现" :back="false">
      <template #content>
        <label class="search">
          <input
            v-model="keyword"
            type="search"
            placeholder="搜索书源"
          />
        </label>
      </template>
    </TitleBar>

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else class="explore-list">
      <EmptyState
        v-if="!filtered.length"
        icon="ic_bottom_explore_e"
        text="当前没有发现源"
        action="去书源管理添加"
        @action="router.push('/book-source')"
      />
      <button
        v-for="source in filtered"
        :key="source.bookSourceUrl"
        class="explore-item"
        @click="open(source)"
      >
        <span class="source-name">{{ source.bookSourceName || source.bookSourceUrl }}</span>
        <span class="source-group">{{ source.bookSourceGroup || "未分组" }}</span>
      </button>
    </main>
  </div>
</template>

<style scoped>
.explore-page {
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
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.explore-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  align-content: start;
}
.explore-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  border: none;
  background: transparent;
  padding: 18px 8px;
  cursor: pointer;
}
.source-name {
  color: var(--ac-primary-text);
  font-size: 13px;
  word-break: break-all;
}
.source-group {
  color: var(--ac-secondary-text);
  font-size: 11px;
}
</style>
