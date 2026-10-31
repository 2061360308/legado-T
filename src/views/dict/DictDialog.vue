<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { DictRule } from "@/core/domain";
import { dictRules } from "@/core/application/rules";
import { buildRequest, extractResult, requestText, sanitizeHtml } from "@/utils/ruleQuery";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean; word: string }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const rules = ref<DictRule[]>([]);
const activeIndex = ref(0);
const result = ref("");
const error = ref("");
const loading = ref(false);
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

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) {
      controller?.abort();
      controller = null;
      return;
    }
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
  <AndroidDialog
    :model-value="modelValue"
    mode="sheet"
    :title="`查词：${word}`"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
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
  </AndroidDialog>
</template>

<style scoped>
.tabs {
  display: flex;
  gap: 2px;
  padding: 0 8px;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
}
.tabs button {
  position: relative;
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 12px 10px;
  font-size: 13px;
  white-space: nowrap;
  cursor: pointer;
}
.tabs button.active {
  color: var(--ac-primary);
}
.tabs button.active::after {
  content: "";
  position: absolute;
  left: 10px;
  right: 10px;
  bottom: 0;
  height: 2px;
  background: var(--ac-primary);
}
.content {
  max-height: 65vh;
  overflow-y: auto;
  padding: 16px 20px;
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
  margin: 0;
  color: var(--ac-secondary-text);
}
.status.error {
  color: var(--ac-error);
}
.empty-tab {
  color: var(--ac-secondary-text);
  font-size: 13px;
  padding: 12px 8px;
}
</style>
