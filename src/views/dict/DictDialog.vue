<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { DictRule } from "@/api/bindings";
import { dictRules } from "@/api/rules";
import { buildRequest, extractResult, requestText, sanitizeHtml } from "@/utils/ruleQuery";

const props = defineProps<{ modelValue: boolean; word: string }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const rules = ref<DictRule[]>([]);
const activeIndex = ref(0);
const result = ref("");
const error = ref("");
const loading = ref(false);
const fullscreen = ref(false);
let controller: AbortController | null = null;

const activeRule = computed(() => rules.value[activeIndex.value] ?? null);

async function loadRules() {
  const all = await dictRules();
  rules.value = all
    .filter((r) => r.enabled !== false)
    .sort((a, b) => (b.sortNumber ?? 0) - (a.sortNumber ?? 0));
}

async function query() {
  const word = props.word.trim();
  controller?.abort();
  controller = null;
  if (!word) {
    result.value = "";
    error.value = "未选中文字";
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
    if (current.signal.aborted) return;
    error.value = String(e);
  } finally {
    if (controller === current) loading.value = false;
  }
}

function selectTab(index: number) {
  if (index === activeIndex.value) return;
  activeIndex.value = index;
  query();
}

function close() {
  controller?.abort();
  controller = null;
  emit("update:modelValue", false);
}

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return;
    fullscreen.value = false;
    await loadRules();
    activeIndex.value = 0;
    await query();
  },
);

watch(
  () => props.word,
  () => {
    if (props.modelValue) query();
  },
);
</script>

<template>
  <div v-if="modelValue" class="mask" @click.self="close">
    <div class="dict-dialog" :class="{ fullscreen }">
      <header>
        <h2>查词：{{ word }}</h2>
        <button @click="fullscreen = !fullscreen">{{ fullscreen ? "还原" : "全屏" }}</button>
        <button @click="close">关闭</button>
      </header>

      <nav class="tabs">
        <button
          v-for="(rule, index) in rules"
          :key="rule.name"
          :class="{ active: index === activeIndex }"
          @click="selectTab(index)"
        >
          {{ rule.name }}
        </button>
        <span v-if="rules.length === 0" class="empty-tab">无启用规则</span>
      </nav>

      <section class="content">
        <p v-if="loading" class="status">查询中…</p>
        <p v-else-if="error" class="status error">{{ error }}</p>
        <div v-else class="html" v-html="result"></div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
  padding: 16px;
}
.dict-dialog {
  background: #faf8f2;
  color: #2f2a24;
  border-radius: 14px;
  width: min(720px, 100%);
  max-height: 84vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.dict-dialog.fullscreen {
  width: 100vw;
  height: 100vh;
  max-height: 100vh;
  border-radius: 0;
}
header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid #e8e2d6;
}
header h2 {
  margin: 0;
  font-size: 16px;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tabs {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  overflow-x: auto;
  border-bottom: 1px solid #e8e2d6;
}
.tabs button {
  white-space: nowrap;
}
.tabs button.active {
  color: #3f51b5;
  border-color: #3f51b5;
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}
.html {
  font-size: 14px;
  line-height: 1.7;
  word-break: break-word;
}
.html :deep(img) {
  max-width: 100%;
}
.status {
  color: #8a8175;
}
.status.error {
  color: #c62828;
}
.empty-tab {
  color: #999;
  font-size: 13px;
}
button {
  border: 1px solid #d8d0c0;
  background: #fffdf8;
  border-radius: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
}
</style>
