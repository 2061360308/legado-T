<script setup lang="ts">
import { ref, watch } from "vue";
import { appLogClear, appLogList } from "@/core/application/logs";
import type { AppLogEntry } from "@/core/domain";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();
const logs = ref<AppLogEntry[]>([]);
const loading = ref(false);
const message = ref("");

async function load() {
  loading.value = true;
  try {
    logs.value = await appLogList(500);
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function clearAll() {
  if (!logs.value.length) return;
  if (!confirm("清空全部日志？")) return;
  await appLogClear();
  logs.value = [];
}

function formatTime(value: number): string {
  return new Date(value).toLocaleString("zh-CN");
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      message.value = "";
      void load();
    }
  },
);
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="日志"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="toolbar">
      <span>{{ logs.length }} 条</span>
      <button :disabled="loading" @click="load">刷新</button>
      <button :disabled="!logs.length" @click="clearAll">清空</button>
    </div>
    <p v-if="message" class="message">{{ message }}</p>
    <p v-else-if="loading" class="message">加载中…</p>
    <div class="logs">
      <article v-for="item in logs" :key="item.id" class="log" :class="item.level">
        <div class="head">
          <span class="level">{{ item.level }}</span>
          <span class="source">{{ item.source }}</span>
          <span class="time">{{ formatTime(item.time) }}</span>
        </div>
        <pre class="body">{{ item.message }}</pre>
      </article>
      <p v-if="!logs.length && !loading" class="empty">暂无日志</p>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--ac-divider);
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.toolbar span {
  flex: 1;
}
.toolbar button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 13px;
}
.logs {
  max-height: 62vh;
  overflow-y: auto;
}
.log {
  padding: 10px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.log.error .level {
  color: var(--ac-error);
}
.log.warn .level {
  color: #e6a23c;
}
.head {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--ac-secondary-text);
  font-size: 11px;
}
.level {
  text-transform: uppercase;
}
.source {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.body {
  margin: 6px 0 0;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--ac-primary-text);
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}
.message,
.empty {
  padding: 16px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
</style>
