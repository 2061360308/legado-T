<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { dictRules } from "@/core/application/rules";
import type { DictRule } from "@/core/domain";
import { buildRequest, extractResult, requestText, sanitizeHtml } from "@/utils/ruleQuery";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const rules = ref<DictRule[]>([]);
const active = ref(0);
const keyword = ref("");
const result = ref("");
const error = ref("");
const loading = ref(false);
let controller: AbortController | null = null;

const activeRule = computed(() => rules.value[active.value] ?? null);

async function loadRules() {
  try {
    rules.value = (await dictRules())
      .filter((rule) => rule.enabled !== false)
      .sort((a, b) => (b.sortNumber ?? 0) - (a.sortNumber ?? 0));
  } catch (e) {
    error.value = String(e);
  }
}

async function query() {
  const word = keyword.value.trim();
  controller?.abort();
  controller = null;
  if (!word) {
    result.value = "";
    error.value = "请输入要查询的词";
    return;
  }
  const rule = activeRule.value;
  if (!rule) {
    result.value = "";
    error.value = "没有启用的字典规则";
    return;
  }
  loading.value = true;
  error.value = "";
  result.value = "";
  const current = new AbortController();
  controller = current;
  try {
    const request = buildRequest(rule.urlRule, { key: word, word, speakText: word });
    const body = await requestText(request, current.signal);
    if (controller !== current) return;
    result.value = sanitizeHtml(extractResult(rule.showRule || "", body));
  } catch (e) {
    if (!current.signal.aborted) error.value = String(e);
  } finally {
    if (controller === current) loading.value = false;
  }
}

function selectRule(index: number) {
  if (active.value === index) return;
  active.value = index;
  if (keyword.value.trim()) void query();
}

onMounted(loadRules);
</script>

<template>
  <div class="dict-page">
    <TitleBar title="字典" @back="$router.back()" />
    <label class="search">
      <Icon name="ic_search_hint" :size="18" />
      <input
        v-model="keyword"
        placeholder="输入词语查询"
        @keydown.enter="query"
      />
      <button :disabled="loading" @click="query">查询</button>
    </label>

    <nav v-if="rules.length" class="tabs">
      <button
        v-for="(rule, index) in rules"
        :key="rule.name"
        :class="{ active: active === index }"
        @click="selectRule(index)"
      >
        {{ rule.name }}
      </button>
    </nav>

    <p v-if="error" class="error">{{ error }}</p>
    <p v-else-if="loading" class="status">查询中…</p>
    <main v-else class="content">
      <div v-if="result" class="html" v-html="result"></div>
      <p v-else class="empty">输入词语后查询</p>
    </main>
  </div>
</template>

<style scoped>
.dict-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.search {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 10px 14px;
  padding: 0 10px;
  border-radius: 18px;
  background: var(--ac-transparent-10);
  color: var(--ac-secondary-text);
}
.search input {
  flex: 1;
  min-width: 0;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  outline: none;
}
.search button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 13px;
  padding: 6px;
}
.tabs {
  display: flex;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
}
.tabs button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 10px 14px;
  font-size: 13px;
  border-bottom: 2px solid transparent;
}
.tabs button.active {
  color: var(--ac-primary);
  border-bottom-color: var(--ac-primary);
}
.error,
.status,
.empty {
  margin: 16px;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.error {
  color: var(--ac-error);
}
.content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 12px 18px 40px;
}
.html {
  color: var(--ac-primary-text);
  font-size: 15px;
  line-height: 1.7;
  word-break: break-word;
}
.html :deep(img) {
  max-width: 100%;
}
</style>
