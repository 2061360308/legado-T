<script setup lang="ts">
import { ref, watch } from "vue";
import type { BookSearchResult } from "@/core/domain";
import { readerSearchBook } from "@/core/application/reader";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{
  modelValue: boolean;
  bookUrl: string;
  initialKeyword?: string;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  jump: [chapterIndex: number, pos: number];
}>();

const keyword = ref("");
const results = ref<BookSearchResult[]>([]);
const loading = ref(false);
const error = ref("");

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      keyword.value = props.initialKeyword ?? "";
      results.value = [];
      error.value = "";
    }
  },
);

async function search() {
  const key = keyword.value.trim();
  if (!key || loading.value) return;
  loading.value = true;
  error.value = "";
  try {
    results.value = await readerSearchBook(props.bookUrl, key);
    if (results.value.length === 0) error.value = "未找到匹配内容";
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function jump(item: BookSearchResult) {
  emit("jump", item.chapterIndex, item.pos);
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="fullscreen"
    title="全书搜索"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="search-row">
      <input
        v-model="keyword"
        placeholder="搜索全书内容"
        autofocus
        @keydown.enter="search"
      />
      <button :disabled="loading" @click="search">{{ loading ? "搜索中" : "搜索" }}</button>
    </div>
    <p v-if="error" class="message">{{ error }}</p>
    <div class="results">
      <button v-for="item in results" :key="`${item.chapterIndex}-${item.pos}`" @click="jump(item)">
        <span class="chapter">{{ item.chapterTitle }}</span>
        <span class="snippet">{{ item.snippet }}</span>
      </button>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.search-row {
  display: flex;
  gap: 8px;
}
.search-row input {
  flex: 1;
  min-width: 0;
  height: 40px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 0 10px;
}
.search-row button {
  flex: none;
  border: none;
  background: var(--ac-primary);
  color: #fff;
  padding: 0 14px;
  font-size: 13px;
}
.message {
  margin: 10px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.results {
  display: flex;
  flex-direction: column;
  margin-top: 8px;
}
.results button {
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
  padding: 10px 4px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.chapter {
  font-size: 13px;
  color: var(--ac-primary);
}
.snippet {
  font-size: 13px;
  line-height: 1.5;
  color: var(--ac-secondary-text);
}
</style>
